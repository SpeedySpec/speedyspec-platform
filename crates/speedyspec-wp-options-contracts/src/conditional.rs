//! What a compare-and-swap `update_option` did (Strategy for writers that read, change and write back an option
//! other processes change too, as cron does with its `cron` option). Optimistic Offline Lock: the version is the
//! stored bytes as read.

/// What [`Options::update_option_if`](crate::ConditionalOptions::update_option_if) did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionalWrite {
    /// Written; every cache update and action `update_option` makes after its write ran.
    Written,
    /// `update_option`'s `false`: nothing to write (the filtered value equals the old one), or a filter refused.
    Unchanged,
    /// Another process wrote first: the stored bytes are not the expected ones. No action after the write fired and
    /// no cache changed. The `update_option` action and the `pre_update_option` filters, which WordPress runs before
    /// its write, have run.
    Conflict,
}
