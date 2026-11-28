declare(strict_types=1);

/**
 * Runs compatibility scenarios and reports what each step did.
 *
 * The same driver runs against WordPress's original code (recording) and against the ports (replaying), so
 * everything it reports is normalised to be independent of the implementation: callbacks are named by their fixture
 * labels, object hashes in callback IDs are replaced by those labels, and arrays keep their order as key/value pairs.
 */
final class SpeedySpecCompat
{
    /** @var array<string, mixed> Callables by fixture label. */
    public static array $callables = [];

    /** @var array<string, array> Fixture specifications by label. */
    public static array $specs = [];

    /** @var array<int, string> Labels by object ID, for closures and fixture objects. */
    public static array $labels = [];

    /** @var array<string, string> Labels by object hash, for callback IDs. */
    public static array $hashes = [];

    /** @var array<string, int> How deeply each recursing fixture is running. */
    public static array $depth = [];

    /** @var list<array> The invocations of the running step. */
    public static array $invocations = [];

    /** @var array<string, mixed> Values kept by steps with `as`. */
    public static array $vars = [];

    /** @var callable|null The area's own snapshot after each step (set by its support file). */
    public static $state = null;

    /** @var callable|null An area's own rewriting of a fixture's arguments before they are reported (set by its support file). */
    public static $arguments = null;

    /** Whether each step reports the hook globals (an area whose behaviour is not hooks may turn it off). */
    public static bool $report_globals = true;

    /** @var string|null A per-request random string reported as `{placeholder}` (wpdb's placeholder escape). */
    public static ?string $placeholder = null;

    /** Whether a string that is not valid UTF-8 is reported as `{"bytes": base64}` (formatting needs exact bytes). */
    public static bool $exact_bytes = false;

    /** @var list<string> The tables each step snapshots unless it names its own (the scenario's `snapshot`). */
    public static array $snapshot = [];

    /** Whether each step reports the cron schedule unless it says otherwise (the scenario's `cron`). */
    public static bool $cron = false;

    /** The connection table snapshots read through, apart from `$wpdb` so they leave no trace in what a step reports. */
    private static ?mysqli $snapshot_db = null;

    /**
     * Creates, or finds again, the callable a fixture specification names.
     */
    public static function callable(array $spec): mixed
    {
        $kind = $spec['kind'] ?? 'closure';
        $label = $spec['label'] ?? ($spec['callback'] . '.' . $kind);
        if (array_key_exists($label, self::$callables)) {
            return self::$callables[$label];
        }
        self::$specs[$label] = $spec;
        $quoted = var_export($label, true);
        $safe = preg_replace('/[^A-Za-z0-9_]/', '_', $label);
        switch ($kind) {
            case 'function':
                $name = "speedyspec_fixture_$safe";
                eval("function $name(...\$args) { return \\SpeedySpecCompat::invoke($quoted, \$args); }");
                $callable = $name;
                break;
            case 'static':
                $class = "SpeedySpecFixture_$safe";
                eval("final class $class { public static function invoke(...\$args) { return \\SpeedySpecCompat::invoke($quoted, \$args); } }");
                $callable = ($spec['as_string'] ?? false) ? "$class::invoke" : [$class, 'invoke'];
                break;
            case 'method':
                $object = new SpeedySpecFixtureObject($label);
                self::label($object, $label);
                $callable = [$object, 'invoke'];
                break;
            case 'invokable':
                $object = new SpeedySpecFixtureObject($label);
                self::label($object, $label);
                $callable = $object;
                break;
            default:
                $callable = static fn (...$args) => self::invoke($label, $args);
                self::label($callable, $label);
        }
        return self::$callables[$label] = $callable;
    }

    public static function label(object $object, string $label): void
    {
        self::$labels[spl_object_id($object)] = $label;
        self::$hashes[spl_object_hash($object)] = $label;
    }

    /**
     * Runs a fixture's behaviour, recording the invocation first.
     */
    public static function invoke(string $label, array $args): mixed
    {
        $spec = self::$specs[$label];
        $with = $spec['with'] ?? [];
        $index = count(self::$invocations);
        $reported = self::$arguments === null ? $args : (self::$arguments)($args);
        self::$invocations[] = ['fixture' => $label, 'args' => self::normalise($reported)];
        $first = $args[0] ?? null;
        switch ($spec['callback']) {
            case 'append':
                return (is_string($first) ? $first : (string) json_encode(self::normalise($first))) . ($with['suffix'] ?? '');
            case 'return_arg':
                return $args[$with['index'] ?? 0] ?? null;
            case 'count':
                return count($args);
            case 'remove_self':
                $removed = remove_filter($with['hook'], self::$callables[$label], $with['priority'] ?? 10);
                self::$invocations[$index]['observed'] = ['removed' => $removed];
                return $first;
            case 'add_callback':
                add_filter($with['hook'], self::callable($with['callback']), $with['priority'] ?? 10, $with['accepted_args'] ?? 1);
                return $first;
            case 'remove_callback':
                $removed = remove_filter($with['hook'], self::callable($with['callback']), $with['priority'] ?? 10);
                self::$invocations[$index]['observed'] = ['removed' => $removed];
                return $first;
            case 'do_action':
                if (self::enter($label, $with)) {
                    do_action($with['hook'], ...self::args($with['args'] ?? []));
                    self::$depth[$label]--;
                }
                return $first;
            case 'apply_filters':
                if (self::enter($label, $with)) {
                    $first = apply_filters($with['hook'], $first, ...array_slice($args, 1));
                    self::$depth[$label]--;
                }
                return $first;
            case 'throw':
                throw new RuntimeException($with['message'] ?? 'thrown by a fixture');
            case 'record_current_filter':
                self::$invocations[$index]['observed'] = [
                    'current_filter' => current_filter(),
                    'doing_filter' => doing_filter(),
                    'doing_hook' => isset($with['hook']) ? doing_filter($with['hook']) : null,
                    'wp_current_filter' => self::normalise($GLOBALS['wp_current_filter'] ?? null),
                ];
                return $first;
            default:
                throw new LogicException('unknown fixture ' . $spec['callback']);
        }
    }

    /**
     * Allows a recursing fixture one more level, up to `max_depth` (1 by default).
     */
    private static function enter(string $label, array $with): bool
    {
        $depth = self::$depth[$label] ?? 0;
        if ($depth >= ($with['max_depth'] ?? 1)) {
            return false;
        }
        self::$depth[$label] = $depth + 1;
        return true;
    }

    /**
     * Turns JSON arguments into PHP values: callback fixtures become callables, `{"var": name}` a kept value.
     */
    public static function args(array $args): array
    {
        $out = [];
        foreach ($args as $key => $arg) {
            $out[$key] = self::arg($arg);
        }
        return $out;
    }

    private static function arg(mixed $arg): mixed
    {
        if (is_array($arg) && isset($arg['bytes']) && is_string($arg['bytes']) && count($arg) === 1) {
            return base64_decode($arg['bytes'], true);
        }
        if (is_array($arg) && isset($arg['callback']) && is_string($arg['callback'])) {
            return self::callable($arg);
        }
        if (is_array($arg) && isset($arg['var']) && count($arg) === 1) {
            return self::$vars[$arg['var']];
        }
        if (is_array($arg) && isset($arg['int']) && count($arg) === 1) {
            return $arg['int'] === 'PHP_INT_MAX' ? PHP_INT_MAX : PHP_INT_MIN;
        }
        if (is_array($arg) && isset($arg['float']) && count($arg) === 1) {
            return (float) $arg['float'];
        }
        if (is_array($arg) && array_key_exists('array', $arg) && count($arg) === 1) {
            $out = [];
            foreach ($arg['array'] as [$key, $value]) {
                if (is_array($key) && isset($key['bytes']) && is_string($key['bytes']) && count($key) === 1) {
                    $key = base64_decode($key['bytes'], true);
                }
                $out[$key] = self::arg($value);
            }
            return $out;
        }
        return is_array($arg) ? self::args($arg) : $arg;
    }

    /**
     * Runs one step and reports it.
     */
    public static function step(array $step): array
    {
        if (isset($step['clock'])) {
            self::freeze((int) $step['clock']);
        }
        self::$invocations = [];
        try {
            if (isset($step['user'])) {
                wp_set_current_user((int) $step['user']);
            }
            $returned = self::normalise(self::call($step['call'], self::args($step['args'] ?? []), $step));
        } catch (Throwable $e) {
            $returned = ['threw' => get_class($e), 'message' => $e->getMessage()];
        }
        $report = ['returned' => $returned, 'invocations' => self::$invocations, 'globals' => self::globals()];
        if (self::$state !== null) {
            $report['state'] = self::normalise((self::$state)());
        }
        $tables = $step['snapshot'] ?? self::$snapshot;
        if ($tables !== []) {
            $report['snapshot'] = self::normalise(self::snapshot($tables));
        }
        if ($step['cron'] ?? self::$cron) {
            $report['cron'] = self::normalise(self::cron());
        }
        return $report;
    }

    /**
     * Freezes PHP's clock at a Unix time: the embedding process decorates `time()`, `microtime()`, `date()`,
     * `gmdate()` and `DateTime` "now" to answer the global this sets (speedyspec-compat's clock module).
     */
    public static function freeze(int $clock): void
    {
        $GLOBALS['speedyspec_compat_clock'] = $clock;
    }

    /**
     * The rows of each table, by table, ordered by its primary key (by every column when it has none).
     *
     * The rows are read through a connection of the driver's own, made from the `DB_*` constants, so a snapshot runs
     * no `query` filter and changes none of `$wpdb`'s counters or last results. Values are as MySQL sends them
     * (strings, or null).
     *
     * @param list<string> $tables
     * @return array<string, list<array<string, string|null>>>
     */
    private static function snapshot(array $tables): array
    {
        $db = self::snapshot_db();
        $out = [];
        foreach ($tables as $table) {
            if (!is_string($table) || !preg_match('/^[A-Za-z0-9_]+$/', $table)) {
                throw new LogicException('a snapshot names a table by its plain name, not ' . json_encode($table));
            }
            $keys = [];
            foreach ($db->query("SHOW KEYS FROM `$table` WHERE Key_name = 'PRIMARY'")->fetch_all(MYSQLI_ASSOC) as $key) {
                $keys[(int) $key['Seq_in_index']] = '`' . $key['Column_name'] . '`';
            }
            ksort($keys);
            if ($keys === []) {
                $count = count($db->query("SHOW COLUMNS FROM `$table`")->fetch_all());
                $keys = range(1, max(1, $count));
            }
            $out[$table] = $db->query("SELECT * FROM `$table` ORDER BY " . implode(', ', $keys))->fetch_all(MYSQLI_ASSOC);
        }
        return $out;
    }

    private static function snapshot_db(): mysqli
    {
        if (self::$snapshot_db !== null) {
            return self::$snapshot_db;
        }
        if (!defined('DB_HOST') || !defined('DB_NAME')) {
            throw new LogicException('a snapshot needs the DB_* constants the area defines for wpdb');
        }
        $host = DB_HOST;
        $port = null;
        $socket = null;
        if (preg_match('/^(.*):(\d+)$/', $host, $m)) {
            [$host, $port] = [$m[1], (int) $m[2]];
        } elseif (preg_match('/^(.*):(\/.+)$/', $host, $m)) {
            [$host, $socket] = [$m[1] === '' ? 'localhost' : $m[1], $m[2]];
        }
        mysqli_report(MYSQLI_REPORT_ERROR | MYSQLI_REPORT_STRICT);
        $db = new mysqli($host, defined('DB_USER') ? DB_USER : '', defined('DB_PASSWORD') ? DB_PASSWORD : '', DB_NAME, $port, $socket);
        mysqli_report(MYSQLI_REPORT_OFF);
        $db->set_charset('utf8mb4');
        return self::$snapshot_db = $db;
    }

    /**
     * The `cron` option's schedule, normalised: one event per entry, ordered by timestamp, hook and argument key,
     * without the option's `version` (WordPress's own `_get_cron_array()` when cron.php is loaded).
     *
     * @return list<array{timestamp: int|string, hook: string, schedule: mixed, interval: mixed, args: mixed}>
     */
    private static function cron(): array
    {
        if (function_exists('_get_cron_array')) {
            $crons = _get_cron_array();
        } elseif (function_exists('get_option')) {
            $crons = get_option('cron');
        } else {
            throw new LogicException('reporting the cron schedule needs the option API');
        }
        $events = [];
        foreach (is_array($crons) ? $crons : [] as $timestamp => $hooks) {
            if ($timestamp === 'version' || !is_array($hooks)) {
                continue;
            }
            foreach ($hooks as $hook => $keyed) {
                foreach (is_array($keyed) ? $keyed : [] as $key => $event) {
                    $events[] = [(int) $timestamp, (string) $hook, (string) $key, [
                        'timestamp' => $timestamp,
                        'hook' => $hook,
                        'schedule' => $event['schedule'] ?? false,
                        'interval' => $event['interval'] ?? null,
                        'args' => $event['args'] ?? [],
                    ]];
                }
            }
        }
        usort($events, static fn (array $a, array $b): int => [$a[0], $a[1], $a[2]] <=> [$b[0], $b[1], $b[2]]);
        return array_column($events, 3);
    }

    private static function call(string $call, array $args, array $step): mixed
    {
        if ($call === 'new WP_Hook') {
            $value = new WP_Hook();
        } elseif (str_starts_with($call, 'get:$')) {
            [$var, $property] = explode('->', substr($call, 5), 2);
            $value = self::target($var)->$property;
        } elseif (str_starts_with($call, 'set:$')) {
            [$var, $property] = explode('->', substr($call, 5), 2);
            self::target($var)->$property = $args[0] ?? null;
            $value = null;
        } elseif (str_starts_with($call, '$')) {
            [$var, $method] = explode('->', substr($call, 1), 2);
            $value = self::target($var)->$method(...$args);
        } elseif (str_starts_with($call, 'global:')) {
            $value = self::global(substr($call, 7), $args);
        } else {
            $value = $call(...$args);
        }
        if (isset($step['as'])) {
            self::$vars[$step['as']] = $value;
            if (is_object($value)) {
                self::label($value, '$' . $step['as']);
            }
        }
        return $value;
    }

    /**
     * Finds the object a `$name` call is on: a value kept with `as`, else the global of that name.
     */
    private static function target(string $var): mixed
    {
        return array_key_exists($var, self::$vars) ? self::$vars[$var] : $GLOBALS[$var];
    }

    /**
     * Reads or writes a global: `read`, `isset`, `count`, `iterate`, `append`, `set`, `unset` or `replace`, with the
     * global's name and a key path (and the value last, when writing).
     */
    private static function global(string $op, array $args): mixed
    {
        $name = array_shift($args);
        $value = in_array($op, ['append', 'set', 'replace'], true) ? array_pop($args) : null;
        if ($op === 'replace') {
            $GLOBALS[$name] = $value;
            return null;
        }
        if ($op === 'unset' || $op === 'set') {
            $last = array_pop($args);
        }
        $ref = &$GLOBALS[$name];
        foreach ($args as $key) {
            if ($op === 'read' || $op === 'isset' || $op === 'count' || $op === 'iterate') {
                if (!isset($ref[$key])) {
                    return $op === 'isset' ? false : ['missing' => $key];
                }
                $copy = $ref[$key];
                unset($ref);
                $ref = $copy;
                continue;
            }
            $ref = &$ref[$key];
        }
        switch ($op) {
            case 'read':
                return $ref;
            case 'isset':
                return isset($ref);
            case 'count':
                return count($ref);
            case 'iterate':
                $seen = [];
                foreach ($ref as $key => $item) {
                    $seen[] = [$key, self::normalise($item)];
                }
                return ['iterated' => $seen];
            case 'append':
                $ref[] = $value;
                return null;
            case 'set':
                $ref[$last] = $value;
                return null;
            case 'unset':
                unset($ref[$last]);
                return null;
        }
        throw new LogicException("unknown global operation $op");
    }

    /**
     * Normalises a value for the report.
     */
    public static function normalise(mixed $value): mixed
    {
        if (is_string($value)) {
            if (self::$exact_bytes && !mb_check_encoding($value, 'UTF-8')) {
                return ['bytes' => base64_encode($value)];
            }
            if (self::$placeholder !== null && self::$placeholder !== '') {
                $value = str_replace(self::$placeholder, '{placeholder}', $value);
            }
            return self::key($value);
        }
        if ($value === null || is_bool($value) || is_int($value)) {
            return $value;
        }
        if (is_float($value)) {
            return is_finite($value) ? ['float' => $value] : ['float' => (string) $value];
        }
        if (is_array($value) || $value instanceof \SpeedySpec\WpHooks\HookGlobal) {
            // A hook global is an array in WordPress and a proxy over the Rust store in the port.
            $pairs = [];
            foreach ($value as $key => $item) {
                $pairs[] = [
                    self::$exact_bytes && is_string($key) && !mb_check_encoding($key, 'UTF-8')
                        ? ['bytes' => base64_encode($key)]
                        : self::key($key),
                    self::normalise($item),
                ];
            }
            return ['array' => $pairs];
        }
        if (is_object($value)) {
            $label = self::$labels[spl_object_id($value)] ?? null;
            if ($value instanceof WP_Hook) {
                return ['WP_Hook' => $label, 'callbacks' => self::normalise($value->callbacks)];
            }
            if ($label !== null) {
                return ['callback' => $label];
            }
            return ['object' => get_class($value)];
        }
        return ['type' => get_debug_type($value)];
    }

    /**
     * Replaces the object identity in a callback ID (an object hash, `spl_object_id:<id>`, or `<id><method>`) with
     * the object's label, so IDs do not depend on how many objects the implementation allocated.
     */
    private static function key(int|string $key): int|string
    {
        if (!is_string($key)) {
            return $key;
        }
        if (strlen($key) >= 32 && isset(self::$hashes[substr($key, 0, 32)])) {
            return '{' . self::$hashes[substr($key, 0, 32)] . '}' . substr($key, 32);
        }
        if (preg_match('/^spl_object_id:(\d+)$/', $key, $m) && isset(self::$labels[(int) $m[1]])) {
            return 'spl_object_id:{' . self::$labels[(int) $m[1]] . '}';
        }
        if (preg_match('/^(\d+)([A-Za-z_][A-Za-z0-9_]*)$/', $key, $m) && isset(self::$labels[(int) $m[1]])) {
            return '{' . self::$labels[(int) $m[1]] . '}' . $m[2];
        }
        return $key;
    }

    private static function globals(): array
    {
        if (!self::$report_globals) {
            return ['wp_filter' => null, 'wp_actions' => null, 'wp_filters' => null, 'wp_current_filter' => null];
        }
        return [
            'wp_filter' => self::normalise($GLOBALS['wp_filter'] ?? null),
            'wp_actions' => self::normalise($GLOBALS['wp_actions'] ?? null),
            'wp_filters' => self::normalise($GLOBALS['wp_filters'] ?? null),
            'wp_current_filter' => self::normalise($GLOBALS['wp_current_filter'] ?? null),
        ];
    }
}

/**
 * A fixture callback that is an object: its `invoke` method, or the object itself through `__invoke`.
 */
final class SpeedySpecFixtureObject
{
    public function __construct(public string $label)
    {
    }

    public function invoke(...$args): mixed
    {
        return SpeedySpecCompat::invoke($this->label, $args);
    }

    public function __invoke(...$args): mixed
    {
        return SpeedySpecCompat::invoke($this->label, $args);
    }
}

if (!function_exists('_deprecated_hook')) {
    /**
     * Records a deprecated hook's notice (WordPress's own is in functions.php, which is not loaded here).
     */
    function _deprecated_hook($hook, $version, $replacement = '', $message = ''): void
    {
        SpeedySpecCompat::$invocations[] = [
            'fixture' => '_deprecated_hook',
            'args' => SpeedySpecCompat::normalise([$hook, $version, $replacement, $message]),
        ];
    }
}

/**
 * Runs a scenario (JSON) and reports its steps (JSON).
 *
 * The scenario may freeze the clock (`clock`), log a user in after the area's setup (`user`), and name the tables
 * every step snapshots (`snapshot`) and whether every step reports the cron schedule (`cron`); a step may override
 * each of them.
 */
function speedyspec_compat_run(string $scenario): string
{
    $scenario = json_decode($scenario, true, 512, JSON_THROW_ON_ERROR);
    if (isset($scenario['clock'])) {
        SpeedySpecCompat::freeze((int) $scenario['clock']);
    }
    SpeedySpecCompat::$snapshot = $scenario['snapshot'] ?? [];
    SpeedySpecCompat::$cron = (bool) ($scenario['cron'] ?? false);
    if (function_exists('speedyspec_compat_area_setup')) {
        speedyspec_compat_area_setup();
    }
    if (isset($scenario['user'])) {
        wp_set_current_user((int) $scenario['user']);
    }
    $report = [];
    foreach ($scenario['steps'] as $step) {
        $report[] = SpeedySpecCompat::step($step);
    }
    return json_encode($report, JSON_THROW_ON_ERROR | JSON_PRESERVE_ZERO_FRACTION | JSON_UNESCAPED_SLASHES | JSON_INVALID_UTF8_SUBSTITUTE);
}
