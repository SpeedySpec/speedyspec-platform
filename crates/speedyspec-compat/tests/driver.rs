//! The driver's write-recording options in the embedded PHP: the frozen clock, `user`, `cron` and `snapshot`. The
//! last two need MySQL (`SPEEDYSPEC_TEST_MYSQL_URL`) and run over WordPress's own hooks and `wpdb` from the pinned
//! release.

use std::process::ExitCode;
use std::rc::Rc;

use serde_json::{Value, json};
use speedyspec_compat::{PhpRunner, Scenario, ScenarioRunner, StepResult};
use speedyspec_hooks_memory::MemoryHooks;
use speedyspec_php::{BootOptions, Php};

/// 2026-01-01 00:00:00 UTC.
const CLOCK: i64 = 1_767_225_600;

/// What each case's request loads before the driver: stand-ins for the WordPress functions the driver calls, and
/// helpers that read the clock the ways WordPress does.
const SETUP: &str = r#"
date_default_timezone_set('America/New_York');
function wp_set_current_user($id) { $GLOBALS['t_user'] = $id; return $id; }
function t_user() { return $GLOBALS['t_user'] ?? 0; }
function t_datetime($tz) { return (new DateTime('now', new DateTimeZone($tz)))->format('Y-m-d H:i:s.u T'); }
function t_default() { $d = new DateTime(); return [$d->getTimestamp(), $d->format('u')]; }
function t_immutable() { return (new DateTimeImmutable('NOW', new DateTimeZone('UTC')))->format('Y-m-d H:i:s.u'); }
function t_create() { return [date_create()->getTimestamp(), date_create_immutable('now')->getTimestamp()]; }
function t_absolute() { return (new DateTime('2020-02-02 02:02:02', new DateTimeZone('UTC')))->getTimestamp(); }
function t_current_time_mysql() {
    return (new DateTime('now', new DateTimeZone('Asia/Tokyo')))->format('Y-m-d H:i:s');
}
// The decorators work through each function's handler, which JIT-compiled code may bypass: the JIT stays off.
function t_jit() {
    $status = function_exists('opcache_get_status') ? @opcache_get_status(false) : false;
    return is_array($status) && !empty($status['jit']['on']);
}
"#;

/// One case: a runner in, a failure out.
type Case = fn(&mut PhpRunner) -> Result<(), String>;

fn runner(php: Php, extra: String) -> PhpRunner {
    PhpRunner::new(php, move |php: &Php| {
        php.eval("speedyspec://test/setup", SETUP)
            .map_err(|e| e.to_string())?;
        if !extra.is_empty() {
            php.eval("speedyspec://test/extra", &extra)
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    })
}

fn run(runner: &mut PhpRunner, scenario: Value) -> Result<Vec<StepResult>, String> {
    let scenario: Scenario = serde_json::from_value(scenario).map_err(|e| e.to_string())?;
    runner.run(&scenario)
}

fn expect(what: &str, got: &Value, want: &Value) -> Result<(), String> {
    if got == want {
        Ok(())
    } else {
        Err(format!("{what}: expected {want}, got {got}"))
    }
}

fn pairs(value: Value) -> Value {
    json!({ "array": value })
}

fn case_clock(runner: &mut PhpRunner) -> Result<(), String> {
    let steps = run(
        runner,
        json!({ "name": "clock", "clock": CLOCK, "steps": [
            { "call": "time" },
            { "call": "microtime" },
            { "call": "microtime", "args": [true] },
            { "call": "date", "args": ["Y-m-d H:i:s T"] },
            { "call": "gmdate", "args": ["Y-m-d H:i:s", null] },
            { "call": "gmdate", "args": ["Y-m-d", 0] },
            { "call": "t_datetime", "args": ["UTC"] },
            { "call": "t_current_time_mysql" },
            { "call": "t_default" },
            { "call": "t_immutable" },
            { "call": "t_create" },
            { "call": "t_absolute" },
            { "call": "time", "clock": CLOCK + 60 },
            { "call": "time" },
            { "call": "microtime", "args": [1] },
            { "call": "t_jit" }
        ] }),
    )?;
    let want = [
        json!(CLOCK),
        json!(format!("0.00000000 {CLOCK}")),
        json!({ "float": CLOCK as f64 }),
        json!("2025-12-31 19:00:00 EST"),
        json!("2026-01-01 00:00:00"),
        json!("1970-01-01"),
        json!("2026-01-01 00:00:00.000000 UTC"),
        json!("2026-01-01 09:00:00"),
        pairs(json!([[0, CLOCK], [1, "000000"]])),
        json!("2026-01-01 00:00:00.000000"),
        pairs(json!([[0, CLOCK], [1, CLOCK]])),
        json!(1_580_608_922),
        json!(CLOCK + 60),
        json!(CLOCK + 60),
        json!({ "float": (CLOCK + 60) as f64 }),
        json!(false),
    ];
    for (i, want) in want.iter().enumerate() {
        expect(&format!("step {i}"), &steps[i].returned, want)?;
    }
    Ok(())
}

fn case_no_clock_reads_the_real_time(runner: &mut PhpRunner) -> Result<(), String> {
    let steps = run(
        runner,
        json!({ "name": "real", "steps": [ { "call": "time" }, { "call": "t_datetime", "args": ["UTC"] } ] }),
    )?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs() as i64;
    let time = steps[0].returned.as_i64().ok_or("time() is not an int")?;
    if (time - now).abs() > 5 {
        return Err(format!(
            "time() without a clock answered {time}, now is {now}"
        ));
    }
    if steps[1].returned == json!("2026-01-01 00:00:00.000000 UTC") {
        return Err("DateTime stayed frozen without a clock".into());
    }
    Ok(())
}

fn case_user(runner: &mut PhpRunner) -> Result<(), String> {
    let steps = run(
        runner,
        json!({ "name": "user", "user": 3, "steps": [
            { "call": "t_user" },
            { "call": "t_user", "user": 5 },
            { "call": "t_user" }
        ] }),
    )?;
    let got: Vec<Value> = steps.iter().map(|s| s.returned.clone()).collect();
    expect("users", &json!(got), &json!([3, 5, 5]))
}

/// WordPress's own hooks and `wpdb` (the `wp-db` area's files) on `url`'s database, with a `wp_options` table holding
/// a `cron` row, and tables to snapshot.
fn wordpress_on(url: &str, root: &str) -> String {
    format!(
        r#"define('ABSPATH', {root:?});
define('WPINC', 'wp-includes');
require ABSPATH . WPINC . '/plugin.php';
require ABSPATH . WPINC . '/load.php';
require ABSPATH . WPINC . '/functions.php';
require ABSPATH . WPINC . '/class-wpdb.php';
$u = parse_url({url:?});
define('DB_NAME', ltrim($u['path'] ?? '', '/'));
define('DB_USER', urldecode($u['user'] ?? ''));
define('DB_PASSWORD', urldecode($u['pass'] ?? ''));
define('DB_HOST', ($u['host'] ?? 'localhost') . (isset($u['port']) ? ':' . $u['port'] : ''));
define('DB_CHARSET', 'utf8mb4');
define('DB_COLLATE', '');
define('WP_DEBUG', false);
$GLOBALS['wpdb'] = new wpdb(DB_USER, DB_PASSWORD, DB_NAME, DB_HOST);
$GLOBALS['wpdb']->set_prefix('wp_');
$m = new mysqli(($u['host'] ?? 'localhost'), DB_USER, DB_PASSWORD, DB_NAME, $u['port'] ?? 3306);
$m->query('DROP TABLE IF EXISTS wp_options');
$m->query("CREATE TABLE wp_options (option_id bigint unsigned NOT NULL auto_increment, option_name varchar(191) NOT NULL default '', option_value longtext NOT NULL, autoload varchar(20) NOT NULL default 'yes', PRIMARY KEY (option_id), UNIQUE KEY option_name (option_name))");
$cron = [
    {clock_late} => ['b_hook' => [md5(serialize(['x'])) => ['schedule' => false, 'args' => ['x']]],
                     'a_hook' => [md5(serialize([])) => ['schedule' => false, 'args' => []]]],
    {clock} => ['z_hook' => [md5(serialize([])) => ['schedule' => 'hourly', 'args' => [], 'interval' => 3600]]],
    'version' => 2,
];
$stmt = $m->prepare("INSERT INTO wp_options (option_name, option_value) VALUES ('cron', ?)");
$serialized = serialize($cron);
$stmt->bind_param('s', $serialized);
$stmt->execute();
$m->query('DROP TABLE IF EXISTS t_compat_snapshot');
$m->query('CREATE TABLE t_compat_snapshot (b int NOT NULL, a int NOT NULL, v varchar(10) NULL, PRIMARY KEY (a, b))');
$m->query("INSERT INTO t_compat_snapshot VALUES (1, 2, 'x'), (2, 1, NULL), (1, 1, 'é')");
$m->query('DROP TABLE IF EXISTS t_compat_nokey');
$m->query('CREATE TABLE t_compat_nokey (v varchar(10) NOT NULL)');
$m->query("INSERT INTO t_compat_nokey VALUES ('b'), ('a'), ('B')");
"#,
        clock = CLOCK,
        clock_late = CLOCK + 10,
    )
}

/// A scenario that counts a `query` filter and reads `$wpdb`'s counters and the hook globals after `observed`, the
/// step that snapshots or reports cron (or not, for the baseline).
fn unperturbed(observed: Value) -> Value {
    let mut middle = json!({ "call": "has_filter", "args": ["query"] });
    for (key, value) in observed.as_object().into_iter().flatten() {
        middle[key] = value.clone();
    }
    json!({ "name": "unperturbed", "steps": [
        { "call": "add_filter", "args": ["query", { "callback": "return_arg", "label": "query" }] },
        { "call": "$wpdb->get_var", "args": ["SELECT 1"] },
        middle,
        { "call": "get:$wpdb->num_queries" },
        { "call": "get:$wpdb->last_query" },
        { "call": "get:$wpdb->last_result" },
        { "call": "$wpdb->get_var", "args": ["SELECT option_value FROM wp_options WHERE option_name = 'cron'"] }
    ] })
}

/// Recording the cron schedule and snapshots leaves what later steps see as it was: no filter ran (the `query`
/// fixture is not invoked and `$wp_filters` counts nothing more), `$wpdb`'s `num_queries`, `last_query` and
/// `last_result` are unchanged, and the `cron` option is not rewritten.
fn case_observing_perturbs_nothing(runner: &mut PhpRunner) -> Result<(), String> {
    let baseline = run(runner, unperturbed(json!({})))?;
    let observed = run(
        runner,
        unperturbed(json!({ "cron": true, "snapshot": ["wp_options", "t_compat_snapshot"] })),
    )?;
    if observed[2].cron.is_null() || observed[2].snapshot.is_null() {
        return Err(format!(
            "the observing step reported nothing: {:?}",
            observed[2]
        ));
    }
    if let Some(threw) = observed[2]
        .cron
        .get("threw")
        .or(observed[2].snapshot.get("threw"))
    {
        return Err(format!("observing failed: {threw}"));
    }
    expect(
        "the query fixture's invocations while observing",
        &json!(observed[2].invocations.len()),
        &json!(0),
    )?;
    for step in 2..baseline.len() {
        let (mut want, mut got) = (baseline[step].clone(), observed[step].clone());
        for result in [&mut want, &mut got] {
            result.cron = Value::Null;
            result.snapshot = Value::Null;
        }
        if want != got {
            return Err(format!(
                "step {step} differs once observed: {want:?} vs {got:?}"
            ));
        }
    }
    Ok(())
}

fn case_cron(runner: &mut PhpRunner) -> Result<(), String> {
    let steps = run(
        runner,
        json!({ "name": "cron", "cron": true, "steps": [
            { "call": "t_user" },
            { "call": "t_user", "cron": false }
        ] }),
    )?;
    let event = |ts: i64, hook: &str, schedule: Value, interval: Value, args: Value| {
        pairs(json!([
            ["timestamp", ts],
            ["hook", hook],
            ["schedule", schedule],
            ["interval", interval],
            ["args", pairs(args)]
        ]))
    };
    expect(
        "ordered by timestamp, then hook, without version",
        &steps[0].cron,
        &pairs(json!([
            [
                0,
                event(CLOCK, "z_hook", json!("hourly"), json!(3600), json!([]))
            ],
            [
                1,
                event(CLOCK + 10, "a_hook", json!(false), Value::Null, json!([]))
            ],
            [
                2,
                event(
                    CLOCK + 10,
                    "b_hook",
                    json!(false),
                    Value::Null,
                    json!([[0, "x"]])
                )
            ]
        ])),
    )?;
    expect("a step that turns cron off", &steps[1].cron, &Value::Null)?;
    let text = serde_json::to_string(&steps[1]).map_err(|e| e.to_string())?;
    if text.contains("\"cron\"") || text.contains("\"snapshot\"") {
        return Err(format!(
            "a step without cron or snapshot serialises them: {text}"
        ));
    }
    Ok(())
}

fn case_snapshot(php_runner: &mut PhpRunner) -> Result<(), String> {
    let steps = run(
        php_runner,
        json!({ "name": "snapshot", "snapshot": ["t_compat_snapshot"], "steps": [
            { "call": "t_user" },
            { "call": "t_user", "snapshot": ["t_compat_nokey", "t_compat_snapshot"] },
            { "call": "t_user", "snapshot": [] },
            { "call": "t_user", "snapshot": ["t_compat_missing"] }
        ] }),
    )?;
    let row = |b: &str, a: &str, v: Value| pairs(json!([["b", b], ["a", a], ["v", v]]));
    let keyed = pairs(json!([
        [0, row("1", "1", json!("é"))],
        [1, row("2", "1", Value::Null)],
        [2, row("1", "2", json!("x"))]
    ]));
    expect(
        "ordered by the primary key",
        &steps[0].snapshot,
        &pairs(json!([["t_compat_snapshot", keyed.clone()]])),
    )?;
    let nokey = pairs(json!([
        [0, pairs(json!([["v", "B"]]))],
        [1, pairs(json!([["v", "a"]]))],
        [2, pairs(json!([["v", "b"]]))]
    ]));
    expect(
        "the step's own tables, in its order; no key orders by bytes",
        &steps[1].snapshot,
        &pairs(json!([
            ["t_compat_nokey", nokey],
            ["t_compat_snapshot", keyed]
        ])),
    )?;
    expect(
        "an empty list snapshots nothing",
        &steps[2].snapshot,
        &Value::Null,
    )?;
    let failed = &steps[3].snapshot;
    expect(
        "a missing table is reported in the step",
        &failed.get("threw").cloned().unwrap_or(Value::Null),
        &json!("LogicException"),
    )?;
    let message = failed.get("message").and_then(Value::as_str).unwrap_or("");
    if !message.contains("t_compat_missing") {
        return Err(format!("the failure does not name the table: {message}"));
    }
    expect(
        "the step itself still returned",
        &steps[3].returned,
        &json!(0),
    )
}

fn main() -> ExitCode {
    let boot = || {
        Php::boot(BootOptions {
            hooks: Rc::new(MemoryHooks::default()),
            preload: &[],
            areas: Vec::new(),
        })
    };
    let php = match boot() {
        Ok(php) => php,
        Err(err) => {
            println!("FAILED: boot: {err}");
            return ExitCode::FAILURE;
        }
    };
    let mysql = std::env::var("SPEEDYSPEC_TEST_MYSQL_URL").ok();
    let extra = match mysql.as_deref() {
        Some(url) => {
            let release = speedyspec_compat::pinned_wordpress();
            match speedyspec_compat::wordpress_release(&release) {
                Ok(root) => wordpress_on(url, &format!("{}/", root.display())),
                Err(why) => {
                    println!("FAILED: WordPress {release}: {why}");
                    return ExitCode::FAILURE;
                }
            }
        }
        None => String::new(),
    };
    let mut runner = runner(php, extra);
    let mut cases: Vec<(&str, Case)> = vec![
        ("clock", case_clock),
        (
            "no_clock_reads_the_real_time",
            case_no_clock_reads_the_real_time,
        ),
        ("user", case_user),
    ];
    if mysql.is_some() {
        cases.push(("cron", case_cron));
        cases.push(("snapshot", case_snapshot));
        cases.push((
            "observing_perturbs_nothing",
            case_observing_perturbs_nothing,
        ));
    } else {
        println!(
            "skipped: cron, snapshot, observing_perturbs_nothing (set SPEEDYSPEC_TEST_MYSQL_URL)"
        );
    }
    let mut failed = 0;
    for (name, case) in cases {
        match case(&mut runner) {
            Ok(()) => println!("ok: {name}"),
            Err(why) => {
                failed += 1;
                println!("FAILED: {name}: {why}");
            }
        }
    }
    if failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
