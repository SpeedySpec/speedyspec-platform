#![doc = include_str!("../README.md")]
//!
//! ## In detail
//!
//! WordPress's options API contracts: the option API other seams call, its errors and the typed reader.

mod autoload;
mod conditional;
mod error;
mod options;
mod typed;
mod value;

pub use autoload::{Autoload, AutoloadRequest, DEFAULT_AUTOLOAD_VALUES};
pub use conditional::ConditionalWrite;
pub use error::OptionsError;
pub use options::{ConditionalOptions, Options};
pub use typed::{FromOptionValue, TypedOptions};
pub use value::{php_empty, php_string};
