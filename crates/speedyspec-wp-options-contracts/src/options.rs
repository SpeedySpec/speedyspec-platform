//! [`Options`]: the option API other seams call (the port; the options seam's `OptionsApi` implements it).

use speedyspec_wp_hooks_contracts::HookValue;

use crate::{ConditionalWrite, OptionsError};

/// WordPress's option functions, one method per `option.php` function, with every filter and action WordPress runs.
///
/// Arguments are PHP's: `$autoload` stays a value (`null`, a bool or a string), because `update_option` treats `null`
/// apart from every other value. The last three methods serve compare-and-swap writers (cron's `cron` option): read
/// the stored bytes, compare them with what was read, and write only where they are unchanged.
pub trait Options {
    /// `get_option( $option, $default_value )`; `default` `None` is a call with one argument.
    fn get_option(&self, name: &str, default: Option<HookValue>)
    -> Result<HookValue, OptionsError>;

    /// `add_option( $option, $value, $deprecated, $autoload )`. Pass `HookValue::Str(String::new())` for
    /// `$deprecated` and `HookValue::Null` for `$autoload` to call it as WordPress's defaults do.
    fn add_option(
        &self,
        name: &str,
        value: HookValue,
        deprecated: HookValue,
        autoload: HookValue,
    ) -> Result<bool, OptionsError>;

    /// `update_option( $option, $value, $autoload )`; `HookValue::Null` is the default `$autoload`.
    fn update_option(
        &self,
        name: &str,
        value: HookValue,
        autoload: HookValue,
    ) -> Result<bool, OptionsError>;

    /// `delete_option( $option )`.
    fn delete_option(&self, name: &str) -> Result<bool, OptionsError>;

    /// `update_option( $option, $value, $autoload )`, whose write happens only where the option's stored bytes are
    /// still `expected` (`None`: the option was absent, so the write inserts and never overwrites).
    ///
    /// A caller whose read disagrees with this process's caches (it expected a row the caches say is missing, or the
    /// reverse) gets [`ConditionalWrite::Conflict`] without a write, and reloads.
    fn update_option_if(
        &self,
        name: &str,
        expected: Option<&[u8]>,
        value: HookValue,
        autoload: HookValue,
    ) -> Result<ConditionalWrite, OptionsError>;

    /// The bytes the repository holds for `name`, with no hook and no cache; `None` when there is no row.
    fn stored_bytes(&self, name: &str) -> Result<Option<Vec<u8>>, OptionsError>;

    /// Drops what this process's object cache holds for `name`: the single option, its slot in `alloptions` and its
    /// `notoptions` entry. The next `get_option` reads the repository.
    fn forget_cached(&self, name: &str) -> Result<(), OptionsError>;

    /// The bytes `update_option` would store for `value` (`maybe_serialize`).
    fn serialized_form(&self, value: &HookValue) -> Result<Vec<u8>, OptionsError>;
}
