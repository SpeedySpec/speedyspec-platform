#![doc = include_str!("../README.md")]
//!
//! ## In detail
//!
//! WordPress's post contracts: the post entity and its values, the post type and post status names and resolved
//! values, the read surface ([`Posts`]), the write surface ([`PostWrites`]), the errors and the ports.

#[cfg(feature = "conformance")]
pub mod conformance;
mod entity;
mod error;
mod ports;
mod read;
mod unported;
mod values;
mod write;

pub use entity::Post;
pub use error::{PostError, PostTypeError, PostWriteError};
pub use ports::{
    BadSlugs, Clock, CurrentPost, PageRow, PostQuery, PostReader, PostRepository, PostStatuses,
    PostTypeSupports, PostTypes, RewriteContext, SlugTruncation, TitleSanitizer, Translator,
};
pub use read::{Filter, LastPostField, MimeFilter, PostRef, PostTypeArg, Posts, Timezone};
pub use unported::{Charset, Comments, Revisions, Rewrite, Taxonomy, Theme, Uploads, Users};
pub use values::{
    CommentStatus, MimeType, PingStatus, PostDate, PostDateTime, PostId, PostParent,
    PostStatusName, PostStatusObject, PostType, PostTypeName, Slug,
};
pub use write::{PostReturn, PostWrites, WriteResult};
