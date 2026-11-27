//! The read surface compiles, converts its errors and stays object-safe.

use std::error::Error as _;
use std::rc::Rc;

use speedyspec_php_serialize::PhpValue;
use speedyspec_wp_hooks_contracts::HookError;
use speedyspec_wp_object_cache_contracts::ObjectCacheError;
use speedyspec_wp_posts_contracts::{
    CurrentPost, Filter, LastPostField, MimeFilter, MimeType, PageRow, Post, PostDateTime,
    PostError, PostId, PostQuery, PostRef, PostRepository, PostStatusName, PostTypeArg,
    PostTypeName, Posts, Slug, Timezone,
};

struct Dummy;

impl Posts for Dummy {
    fn get(&self, id: PostRef, _: Filter) -> Result<Option<Post>, PostError> {
        Ok(match id {
            PostRef::Id(id) => Some(Post {
                id,
                ..Post::default()
            }),
            PostRef::Current => None,
        })
    }
    fn field(&self, _: &str, _: PostRef, _: Filter) -> Result<PhpValue, PostError> {
        Ok(PhpValue::Str(Vec::new()))
    }
    fn status(&self, _: PostRef) -> Result<Option<PostStatusName>, PostError> {
        Ok(Some(PostStatusName::new("publish")))
    }
    fn post_type(&self, _: PostRef) -> Result<Option<PostTypeName>, PostError> {
        Ok(None)
    }
    fn mime_type(&self, _: PostRef) -> Result<Option<MimeType>, PostError> {
        Ok(None)
    }
    fn ancestors(&self, _: PostRef) -> Result<Vec<PostId>, PostError> {
        Ok(vec![PostId(2), PostId(1)])
    }
    fn parent_id(&self, _: PostRef) -> Result<Option<PostId>, PostError> {
        Ok(Some(PostId(2)))
    }
    fn page_by_path(&self, _: &str, post_type: &PostTypeArg) -> Result<Option<Post>, PostError> {
        Ok(Some(Post {
            post_type: post_type.types()[0].clone(),
            ..Post::default()
        }))
    }
    fn page_uri(&self, _: PostRef) -> Result<Option<Vec<u8>>, PostError> {
        Ok(Some(b"parent/child".to_vec()))
    }
    fn all_page_ids(&self) -> Result<Vec<PostId>, PostError> {
        Ok(Vec::new())
    }
    fn count_posts(&self, _: &PostTypeName, _: &str) -> Result<PhpValue, PostError> {
        Ok(PhpValue::Object {
            class: "stdClass".to_string(),
            properties: Vec::new(),
        })
    }
    fn count_attachments(&self, mime_type: Option<&MimeFilter>) -> Result<PhpValue, PostError> {
        Ok(PhpValue::Bool(mime_type.is_some()))
    }
    fn last_post_date(&self, timezone: &str, _: &PostTypeName) -> Result<PhpValue, PostError> {
        Ok(match Timezone::parse(timezone) {
            Some(_) => PhpValue::Str(PostDateTime::ZERO.as_str().as_bytes().to_vec()),
            None => PhpValue::Bool(false),
        })
    }
    fn last_post_modified(&self, _: &str, _: &PostTypeName) -> Result<PhpValue, PostError> {
        Ok(PhpValue::Bool(false))
    }
    fn available_mime_types(&self, _: &PostTypeName) -> Result<Vec<MimeType>, PostError> {
        Ok(Vec::new())
    }
    fn update_cache(&self, _: &[Post]) -> Result<(), PostError> {
        Ok(())
    }
    fn clean_cache(&self, _: PostId) -> Result<(), PostError> {
        Err(HookError::message("boom").into())
    }
    fn prime_caches(&self, _: &[PostId], _: bool, _: bool) -> Result<(), PostError> {
        Ok(())
    }
    fn prime_parent_caches(&self, _: &[PostId]) -> Result<(), PostError> {
        Ok(())
    }
}

impl PostRepository for Dummy {
    fn engine(&self) -> String {
        "dummy".to_string()
    }
    fn find(&self, _: PostId) -> Result<Option<Post>, PostError> {
        Ok(None)
    }
    fn insert(&self, _: &Post) -> Result<Option<PostId>, PostError> {
        Ok(None)
    }
    fn attachment_slug_owner(&self, _: &Slug, _: PostId) -> Result<Option<Slug>, PostError> {
        Ok(None)
    }
    fn hierarchical_slug_owner(
        &self,
        _: &Slug,
        _: &PostTypeName,
        _: PostId,
        _: PostId,
    ) -> Result<Option<Slug>, PostError> {
        Ok(None)
    }
    fn flat_slug_owner(
        &self,
        _: &Slug,
        _: &PostTypeName,
        _: PostId,
    ) -> Result<Option<Slug>, PostError> {
        Ok(None)
    }
    fn find_many(&self, _: &[PostId]) -> Result<Vec<Post>, PostError> {
        Ok(Vec::new())
    }
    fn parent_ids(&self, ids: &[PostId]) -> Result<Vec<(PostId, PostId)>, PostError> {
        Ok(ids.iter().map(|&id| (id, PostId(0))).collect())
    }
    fn pages_by_names(&self, _: &[Slug], _: &[PostTypeName]) -> Result<Vec<PageRow>, PostError> {
        Ok(vec![PageRow {
            id: PostId(3),
            name: Slug::from("about"),
            parent: PostId(0),
            post_type: PostTypeName::new("page"),
        }])
    }
    fn all_page_ids(&self) -> Result<Vec<PostId>, PostError> {
        Ok(Vec::new())
    }
    fn count_by_status(
        &self,
        _: &PostTypeName,
        _: Option<u64>,
    ) -> Result<Vec<(PostStatusName, u64)>, PostError> {
        Ok(Vec::new())
    }
    fn count_attachments(
        &self,
        _: Option<&MimeFilter>,
    ) -> Result<(Vec<(MimeType, u64)>, u64), PostError> {
        Ok((Vec::new(), 0))
    }
    fn last_post_time(
        &self,
        _: LastPostField,
        _: &[PostTypeName],
        _: Timezone,
    ) -> Result<Option<PostDateTime>, PostError> {
        Ok(None)
    }
    fn available_mime_types(&self, _: &PostTypeName) -> Result<Vec<MimeType>, PostError> {
        Ok(Vec::new())
    }
}

impl CurrentPost for Dummy {
    fn current(&self) -> Option<PhpValue> {
        Some(PhpValue::Int(5))
    }
}

impl PostQuery for Dummy {
    fn query(&self, _: PhpValue) -> Result<PhpValue, PostError> {
        Ok(PhpValue::Array(Vec::new()))
    }
}

#[test]
fn the_traits_are_object_safe_and_callable() {
    let posts: Rc<dyn Posts> = Rc::new(Dummy);
    let repo: Rc<dyn PostRepository> = Rc::new(Dummy);
    let current: Rc<dyn CurrentPost> = Rc::new(Dummy);
    let query: Rc<dyn PostQuery> = Rc::new(Dummy);

    let post = posts.get(PostRef::Id(PostId(7)), Filter::Raw).unwrap();
    assert_eq!(post.map(|p| p.id), Some(PostId(7)));
    assert!(
        posts
            .get(PostRef::Current, Filter::Display)
            .unwrap()
            .is_none()
    );
    assert!(matches!(
        posts.clean_cache(PostId(1)),
        Err(PostError::Hook(_))
    ));
    assert_eq!(
        repo.parent_ids(&[PostId(4)]).unwrap(),
        vec![(PostId(4), PostId(0))]
    );
    assert_eq!(repo.pages_by_names(&[], &[]).unwrap()[0].id, PostId(3));
    assert_eq!(current.current(), Some(PhpValue::Int(5)));
    assert!(query.query(PhpValue::Null).is_ok());
    assert_eq!(
        posts
            .last_post_date("UTC", &PostTypeName::new("any"))
            .unwrap(),
        PhpValue::Bool(false)
    );
    let filter = MimeFilter::Many(vec!["image".to_string()]);
    assert_eq!(
        posts.count_attachments(Some(&filter)).unwrap(),
        PhpValue::Bool(true)
    );
    assert_eq!(
        repo.count_by_status(&PostTypeName::new("post"), Some(7))
            .unwrap(),
        vec![]
    );
}

#[test]
fn a_single_post_type_also_searches_attachments() {
    let page = PostTypeName::new("page");
    assert_eq!(
        PostTypeArg::One(page.clone()).types(),
        vec![page.clone(), PostTypeName::new("attachment")]
    );
    assert_eq!(
        PostTypeArg::Many(vec![page.clone()]).types(),
        vec![page.clone()]
    );
    assert_eq!(
        PostTypeArg::Many(Vec::new()).types(),
        Vec::<PostTypeName>::new()
    );
    assert_ne!(
        PostTypeArg::One(page.clone()),
        PostTypeArg::Many(vec![page])
    );
    let posts: Rc<dyn Posts> = Rc::new(Dummy);
    let found = posts
        .page_by_path("about", &PostTypeArg::One(PostTypeName::new("book")))
        .unwrap();
    assert_eq!(found.map(|p| p.post_type), Some(PostTypeName::new("book")));
}

#[test]
fn errors_convert_into_their_variants() {
    let hook = PostError::from(HookError::message("boom"));
    assert!(matches!(hook, PostError::Hook(_)));
    assert_eq!(hook.to_string(), "a hook failed: boom");
    assert!(hook.source().is_some());

    let cache = PostError::from(ObjectCacheError::Unsupported("k".into()));
    assert!(matches!(cache, PostError::Cache(_)));
    assert!(cache.source().is_some());

    assert!(PostError::Db("x".into()).source().is_none());
}

#[test]
fn a_hook_error_keeps_its_payload() {
    let payload: Rc<dyn std::any::Any> = Rc::new(42_i32);
    let error = PostError::from(HookError {
        message: "thrown".into(),
        payload: Some(payload),
    });
    let PostError::Hook(e) = error else {
        panic!("not a hook error");
    };
    let kept = e.payload.expect("the payload travels");
    assert_eq!(kept.downcast_ref::<i32>(), Some(&42));
}

#[test]
fn the_small_enums_parse_as_wordpress_compares() {
    for (text, filter) in [
        ("raw", Filter::Raw),
        ("edit", Filter::Edit),
        ("db", Filter::Db),
        ("display", Filter::Display),
        ("attribute", Filter::Attribute),
        ("js", Filter::Js),
    ] {
        assert_eq!(Filter::parse(text), Some(filter));
        assert_eq!(filter.as_str(), text);
    }
    assert_eq!(Filter::parse("Raw"), None);
    assert_eq!(Filter::parse("rss"), None);

    assert_eq!(LastPostField::parse("date"), Some(LastPostField::Date));
    assert_eq!(
        LastPostField::parse("modified"),
        Some(LastPostField::Modified)
    );
    assert_eq!(LastPostField::parse("Date"), None);
    assert_eq!(LastPostField::Modified.as_str(), "modified");

    // `_get_last_post_time()` lowercases the timezone first.
    assert_eq!(Timezone::parse("GMT"), Some(Timezone::Gmt));
    assert_eq!(Timezone::parse("blog"), Some(Timezone::Blog));
    assert_eq!(Timezone::parse("Server"), Some(Timezone::Server));
    assert_eq!(Timezone::parse("utc"), None);
    assert_eq!(Timezone::Server.as_str(), "server");
}
