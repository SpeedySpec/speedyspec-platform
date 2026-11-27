#![doc = include_str!("../README.md")]
//!
//! ## In detail
//!
//! WordPress's post contracts: the post entity and its values, the post type and post status names and resolved
//! values, the read surface ([`Posts`]), the errors and the ports.

#[cfg(feature = "conformance")]
pub mod conformance;
mod entity;
mod error;
mod ports;
mod read;
mod values;

pub use entity::Post;
pub use error::{PostError, PostTypeError};
pub use ports::{
    BadSlugs, Clock, CurrentPost, PageRow, PostQuery, PostReader, PostRepository, PostStatuses,
    PostTypeSupports, PostTypes, RewriteContext, SlugTruncation, TitleSanitizer, Translator,
};
pub use read::{Filter, LastPostField, MimeFilter, PostRef, PostTypeArg, Posts, Timezone};
pub use values::{
    CommentStatus, MimeType, PingStatus, PostDate, PostDateTime, PostId, PostStatusName,
    PostStatusObject, PostType, PostTypeName, Slug,
};
