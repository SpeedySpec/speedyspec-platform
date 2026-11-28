//! The driver's write-recording options in the embedded PHP: the frozen clock, `user`, `cron` and `snapshot` (the
//! last only with MySQL, through `SPEEDYSPEC_TEST_MYSQL_URL`).

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
function _get_cron_array() { return $GLOBALS['t_cron'] ?? []; }
function t_schedule($timestamp, $hook, $args, $schedule = false) {
    $key = md5(serialize($args));
    $event = ['schedule' => $schedule, 'args' => $args];
    if ($schedule !== false) { $event['interval'] = 3600; }
    $GLOBALS['t_cron'][$timestamp][$hook][$key] = $event;
    return true;
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
            { "call": "time" }
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

fn case_cron(runner: &mut PhpRunner) -> Result<(), String> {
    let steps = run(
        runner,
        json!({ "name": "cron", "cron": true, "steps": [
            { "call": "t_schedule", "args": [CLOCK + 10, "b_hook", ["x"]] },
            { "call": "t_schedule", "args": [CLOCK, "z_hook", [], "hourly"] },
            { "call": "t_schedule", "args": [CLOCK + 10, "a_hook", []], "cron": false },
            { "call": "t_user" }
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
        "after one event",
        &steps[0].cron,
        &pairs(json!([[
            0,
            event(
                CLOCK + 10,
                "b_hook",
                json!(false),
                Value::Null,
                json!([[0, "x"]])
            )
        ]])),
    )?;
    expect("a step that turns cron off", &steps[2].cron, &Value::Null)?;
    expect(
        "ordered by timestamp, then hook",
        &steps[3].cron,
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
    let text = serde_json::to_string(&steps[2]).map_err(|e| e.to_string())?;
    if text.contains("\"cron\"") || text.contains("\"snapshot\"") {
        return Err(format!(
            "a step without cron or snapshot serialises them: {text}"
        ));
    }
    Ok(())
}

/// The DB_* constants for `url`, as an area's setup defines them for wpdb.
fn db_constants(url: &str) -> String {
    format!(
        r#"$u = parse_url({url:?});
define('DB_NAME', ltrim($u['path'] ?? '', '/'));
define('DB_USER', urldecode($u['user'] ?? ''));
define('DB_PASSWORD', urldecode($u['pass'] ?? ''));
define('DB_HOST', ($u['host'] ?? 'localhost') . (isset($u['port']) ? ':' . $u['port'] : ''));
$m = new mysqli(($u['host'] ?? 'localhost'), DB_USER, DB_PASSWORD, DB_NAME, $u['port'] ?? 3306);
$m->query('DROP TABLE IF EXISTS t_compat_snapshot');
$m->query('CREATE TABLE t_compat_snapshot (b int NOT NULL, a int NOT NULL, v varchar(10) NULL, PRIMARY KEY (a, b))');
$m->query("INSERT INTO t_compat_snapshot VALUES (1, 2, 'x'), (2, 1, NULL), (1, 1, 'é')");
$m->query('DROP TABLE IF EXISTS t_compat_nokey');
$m->query('CREATE TABLE t_compat_nokey (v varchar(10) NOT NULL)');
$m->query("INSERT INTO t_compat_nokey VALUES ('b'), ('a')");
function t_insert($v) {{
    $m = new mysqli(parse_url(getenv('SPEEDYSPEC_TEST_MYSQL_URL'), PHP_URL_HOST), DB_USER, DB_PASSWORD, DB_NAME,
        parse_url(getenv('SPEEDYSPEC_TEST_MYSQL_URL'), PHP_URL_PORT) ?: 3306);
    return $m->query("INSERT INTO t_compat_nokey VALUES ('$v')");
}}
"#
    )
}

fn case_snapshot(php_runner: &mut PhpRunner) -> Result<(), String> {
    let steps = run(
        php_runner,
        json!({ "name": "snapshot", "snapshot": ["t_compat_snapshot"], "steps": [
            { "call": "t_user" },
            { "call": "t_insert", "args": ["c"], "snapshot": ["t_compat_nokey", "t_compat_snapshot"] },
            { "call": "t_user", "snapshot": [] }
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
        [0, pairs(json!([["v", "a"]]))],
        [1, pairs(json!([["v", "b"]]))],
        [2, pairs(json!([["v", "c"]]))]
    ]));
    expect(
        "the step's own tables, in its order",
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
    let extra = mysql.as_deref().map(db_constants).unwrap_or_default();
    let mut runner = runner(php, extra);
    let mut cases: Vec<(&str, Case)> = vec![
        ("clock", case_clock),
        (
            "no_clock_reads_the_real_time",
            case_no_clock_reads_the_real_time,
        ),
        ("user", case_user),
        ("cron", case_cron),
    ];
    if mysql.is_some() {
        cases.push(("snapshot", case_snapshot));
    } else {
        println!("skipped: snapshot (set SPEEDYSPEC_TEST_MYSQL_URL)");
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
