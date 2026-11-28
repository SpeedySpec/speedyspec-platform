//! The read surface compiles, converts its errors and stays object-safe.

use std::error::Error as _;
use std::rc::Rc;

use speedyspec_php_serialize::PhpValue;
use speedyspec_wp_hooks_contracts::HookError;
use speedyspec_wp_hooks_contracts::HookValue;
use speedyspec_wp_object_cache_contracts::ObjectCacheError;
use speedyspec_wp_posts_contracts::{
    BadSlugs, ColumnFormat, INT_FIELDS, PostRow, RowValue, WriteOutcome, column_format,
};
use speedyspec_wp_posts_contracts::{
    Clock, CurrentPost, Filter, LastPostField, MimeFilter, MimeType, PageRow, Post, PostDateTime,
    PostError, PostId, PostParent, PostQuery, PostRef, PostRepository, PostStatusName, PostTypeArg,
    PostTypeName, Posts, RewriteContext, Slug, Timezone, Translator,
};
use speedyspec_wp_posts_contracts::{
    PostArg, PostObject, PostReturn, PostWriteError, PostWrites, WriteResult,
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
    fn insert(&self, row: &PostRow) -> Result<WriteOutcome<PostId>, PostError> {
        // As MySQL does: an explicit `ID` above 0 is kept, 0 and below take the next id (50 here), and `NULL` or an
        // unknown column is rejected.
        if row
            .columns()
            .iter()
            .any(|(name, value)| *value == RowValue::Null || name == b"bogus")
        {
            return Ok(WriteOutcome::failed(
                "Unknown column 'bogus' in 'field list'",
            ));
        }
        Ok(WriteOutcome::Written(match row.get("ID") {
            Some(RowValue::Int(id)) if *id > 0 => PostId(*id as u64),
            _ => PostId(50),
        }))
    }
    fn attachment_slug_owner(&self, _: &Slug, _: PostId) -> Result<Option<Slug>, PostError> {
        Ok(None)
    }
    fn hierarchical_slug_owner(
        &self,
        _: &Slug,
        _: &PostTypeName,
        _: PostId,
        _: PostParent,
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
    fn update(&self, _: PostId, row: &PostRow) -> Result<WriteOutcome<u64>, PostError> {
        Ok(if row.is_empty() {
            WriteOutcome::failed("You have an error in your SQL syntax")
        } else {
            WriteOutcome::Written(1)
        })
    }
    fn update_name(&self, _: PostId, _: &Slug) -> Result<Option<u64>, PostError> {
        Ok(Some(1))
    }
    fn update_guid(&self, _: PostId, _: &[u8]) -> Result<Option<u64>, PostError> {
        Ok(Some(0))
    }
    fn update_status(&self, _: PostId, _: &PostStatusName) -> Result<Option<u64>, PostError> {
        Ok(Some(1))
    }
    fn update_pinged(&self, _: PostId, _: &[u8]) -> Result<Option<u64>, PostError> {
        Ok(None)
    }
    fn update_post_type(&self, id: i64, _: &PostTypeName) -> Result<Option<u64>, PostError> {
        Ok(Some(u64::from(id > 0)))
    }
    fn import_id_taken(&self, id: i64) -> Result<bool, PostError> {
        Ok(id == 1)
    }
    fn raw_row(&self, _: PostId) -> Result<Option<Post>, PostError> {
        Ok(None)
    }
    fn children_of_type(&self, _: PostId, _: &PostTypeName) -> Result<Vec<Post>, PostError> {
        Ok(Vec::new())
    }
    fn reparent_children(
        &self,
        _: PostId,
        _: PostParent,
        _: &PostTypeName,
    ) -> Result<Option<u64>, PostError> {
        Ok(Some(2))
    }
    fn revision_ids(&self, _: PostId) -> Result<Vec<PostId>, PostError> {
        Ok(vec![PostId(9)])
    }
    fn reparent_attachments(&self, _: PostId, _: PostParent) -> Result<Option<u64>, PostError> {
        Ok(Some(0))
    }
    fn remove(&self, _: PostId) -> Result<Option<u64>, PostError> {
        Ok(Some(1))
    }
    fn auto_drafts_older_than_a_week(&self) -> Result<Vec<PostId>, PostError> {
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
    fn get_posts(&self, args: PhpValue) -> Result<PhpValue, PostError> {
        Ok(args)
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
    assert_eq!(query.get_posts(PhpValue::Int(3)).unwrap(), PhpValue::Int(3));
    assert!(repo.import_id_taken(1).unwrap());
    assert_eq!(
        repo.reparent_children(PostId(4), PostParent::Int(-1), &PostTypeName::new("page"))
            .unwrap(),
        Some(2)
    );
    assert_eq!(repo.revision_ids(PostId(4)).unwrap(), vec![PostId(9)]);
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

#[test]
fn an_import_id_is_present_or_absent_and_signed() {
    // `wp_insert_post()` casts a non-empty `import_id` with `(int)`: `'abc'` is `Some(0)` and `-5` is `Some(-5)`; both
    // are checked (`WHERE ID = 0`, `WHERE ID = -5`) and sent as the `ID`. An empty one is `None`.
    let repo: Rc<dyn PostRepository> = Rc::new(Dummy);
    let row = PostRow::from_post(&Post::default());
    let with_id = |id: i64| row.clone().with("ID", RowValue::Int(id));
    assert_eq!(repo.insert(&row).unwrap().written(), Some(PostId(50)));
    assert_eq!(
        repo.insert(&with_id(0)).unwrap().written(),
        Some(PostId(50))
    );
    assert_eq!(
        repo.insert(&with_id(-5)).unwrap().written(),
        Some(PostId(50))
    );
    assert_eq!(
        repo.insert(&with_id(100)).unwrap().written(),
        Some(PostId(100))
    );
    // `ID` is appended after `compact()`'s columns.
    assert_eq!(with_id(100).columns().last().unwrap().0, b"ID");
    assert!(!repo.import_id_taken(0).unwrap());
    assert!(!repo.import_id_taken(-5).unwrap());
    // `set_post_type()`'s raw id reaches the statement: `WHERE `ID` = -3` changes nothing.
    assert_eq!(
        repo.update_post_type(-3, &PostTypeName::new("page"))
            .unwrap(),
        Some(0)
    );
}

#[test]
fn a_null_parent_is_representable() {
    // A `wp_insert_post_parent` callback returned null: the insert carries `NULL` and MySQL rejects it (1048).
    let repo: Rc<dyn PostRepository> = Rc::new(Dummy);
    let post = Post {
        parent: PostParent::Null,
        ..Post::default()
    };
    let row = PostRow::from_post(&post);
    assert_eq!(row.get("post_parent"), Some(&RowValue::Null));
    assert_eq!(repo.insert(&row).unwrap().written(), None);
    assert_eq!(post.parent.prepared(), 0);
    assert!(
        repo.hierarchical_slug_owner(
            &Slug::from("a"),
            &PostTypeName::new("page"),
            PostId(0),
            PostParent::Null
        )
        .unwrap()
        .is_none()
    );
}

#[test]
fn a_post_object_keeps_its_php_handle() {
    // Hooks share the caller's object: the handle travels through the argument and back out as the same `Rc`.
    let object: Rc<dyn std::any::Any> = Rc::new("WP_Post #7");
    let post = PostObject::with_handle(
        Post {
            id: PostId(7),
            ..Post::default()
        },
        HookValue::Opaque(object.clone()),
    );
    let arg = PostArg::from(post.clone());
    let PostArg::Object(passed) = &arg else {
        panic!("not an object");
    };
    let Some(HookValue::Opaque(handle)) = &passed.handle else {
        panic!("the handle is lost");
    };
    assert!(Rc::ptr_eq(handle, &object));

    let returned = PostReturn::Post(Box::new(post));
    let PostReturn::Post(back) = returned else {
        panic!("not a post");
    };
    assert_eq!(back.post.id, PostId(7));
    assert!(matches!(&back.handle, Some(HookValue::Opaque(h)) if Rc::ptr_eq(h, &object)));

    // With no PHP object yet the binding builds one; an id stays a reference.
    assert_eq!(PostObject::new(Post::default()).handle, None);
    assert_eq!(
        PostArg::from(PostRef::Current),
        PostArg::Ref(PostRef::Current)
    );
}

#[test]
fn a_written_id_is_an_integer_post_id() {
    // `sanitize_post( $postarr, 'db' )` casts `ID` with `(int)` before and after its filters, so the id
    // `wp_update_post( array( 'ID' => '8' ) )` returns is the int 8, which a `PostId` carries whole.
    let saved: WriteResult = Ok(PostId(8));
    let zero: WriteResult = Ok(PostId(0));
    let failed: WriteResult = Err(PostWriteError::InvalidPost);
    assert_eq!(saved, Ok(PostId(8)));
    assert_eq!(zero, Ok(PostId(0)));
    assert!(failed.is_err());
}

#[test]
fn a_post_object_carries_the_properties_wordpress_sets_on_it() {
    // `wp_insert_post()` sets `$post->page_template` on the object its later hooks share (post.php 5402).
    let object = PostObject::new(Post::default())
        .with_property("page_template", HookValue::Str("tpl.php".to_string()))
        .with_property("extra", HookValue::Int(1))
        .with_property("page_template", HookValue::Str("default".to_string()));
    // Set again, it keeps its first position, as a PHP property does.
    assert_eq!(
        object.properties,
        vec![
            (
                "page_template".to_string(),
                HookValue::Str("default".to_string())
            ),
            ("extra".to_string(), HookValue::Int(1)),
        ]
    );
    assert_eq!(object.property("extra"), Some(&HookValue::Int(1)));
    assert_eq!(object.property("missing"), None);
    assert!(object.handle.is_none());

    // A handle keeps its properties empty until WordPress sets one.
    let handled = PostObject::with_handle(Post::default(), HookValue::Int(9));
    assert!(handled.properties.is_empty());
    assert_eq!(handled.handle, Some(HookValue::Int(9)));
}

#[test]
fn transition_hands_back_the_object_its_hooks_received() {
    // The use case keeps that object for the hooks it fires afterwards, so one call shares one object.
    let transition: fn(
        &dyn PostWrites,
        &PostStatusName,
        &PostStatusName,
        &PostObject,
    ) -> Result<PostObject, PostError> = |writes, new, old, post| writes.transition(new, old, post);
    let _ = transition;
}

#[test]
fn a_null_or_negative_parent_reaches_the_statements_as_an_integer() {
    // The known divergence: a non-integer `wp_insert_post_parent` return is carried as its `intval()`, which is what
    // `%d` sends; the later filters' raw value is the use case's to keep.
    assert_eq!(PostParent::Int(7).prepared(), 7);
    assert_eq!(PostParent::Null.prepared(), 0);
    assert_eq!(PostParent::Int(-5).stored(), PostParent::NONE);
}

#[cfg(feature = "conformance")]
#[test]
fn the_read_and_write_suites_run_apart() {
    // An engine without MySQL's `sql_mode` behaviour (SQLite) runs the read suite alone; the full suite runs both.
    use speedyspec_wp_posts_contracts::conformance::{
        post_repository_suite, read_suite, write_suite,
    };
    let suites: [fn(&dyn PostRepository); 3] = [read_suite, write_suite, post_repository_suite];
    assert_eq!(suites.len(), 3);
}

#[test]
fn a_row_is_the_filtered_data_in_its_order() {
    // `compact()`'s 21 columns, in its order, with no `ID` and no `comment_count`.
    let row = PostRow::from_post(&Post::default());
    let names: Vec<&str> = row
        .columns()
        .iter()
        .map(|(n, _)| std::str::from_utf8(n).unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "post_author",
            "post_date",
            "post_date_gmt",
            "post_content",
            "post_content_filtered",
            "post_title",
            "post_excerpt",
            "post_status",
            "post_type",
            "comment_status",
            "ping_status",
            "post_password",
            "post_name",
            "to_ping",
            "pinged",
            "post_modified",
            "post_modified_gmt",
            "post_parent",
            "menu_order",
            "post_mime_type",
            "guid",
        ]
    );
    assert_eq!(row.get("post_author"), Some(&RowValue::Int(0)));
    assert_eq!(
        row.get("post_status"),
        Some(&RowValue::Str(b"publish".to_vec()))
    );

    // A callback drops a column and adds one: the dropped one is gone, the added one goes last; setting a key again
    // keeps its place, as PHP's assignment does.
    let mut filtered = row.clone();
    filtered.remove("post_content");
    filtered.set("bogus", RowValue::Str(b"x".to_vec()));
    filtered.set("post_title", RowValue::Str(b"Again".to_vec()));
    assert_eq!(filtered.get("post_content"), None);
    assert_eq!(filtered.columns().len(), 21);
    assert_eq!(filtered.columns().last().unwrap().0, b"bogus");
    assert_eq!(filtered.columns()[4].0, b"post_title");

    let repo: Rc<dyn PostRepository> = Rc::new(Dummy);
    assert_eq!(repo.insert(&filtered).unwrap().written(), None);
    assert_eq!(
        repo.update(PostId(3), &PostRow::new()).unwrap().written(),
        None
    );
    assert_eq!(
        repo.update(
            PostId(3),
            &PostRow::new().with("comment_count", RowValue::Int(2))
        )
        .unwrap()
        .written(),
        Some(1)
    );
}

#[test]
fn a_column_takes_wpdbs_placeholder() {
    for name in [
        "post_author",
        "post_parent",
        "menu_order",
        "ID",
        "comment_count",
        "post_id",
    ] {
        assert_eq!(column_format(name), ColumnFormat::Int, "{name}");
    }
    // Exact names only, as `isset( $wpdb->field_types[ $field ] )`.
    for name in ["post_title", "guid", "id", "bogus", ""] {
        assert_eq!(column_format(name), ColumnFormat::Str, "{name}");
    }
    // Keys are bytes, as PHP's are; a non-UTF-8 key names no `%d` field.
    assert_eq!(column_format(b"ID".as_slice()), ColumnFormat::Int);
    assert_eq!(column_format([0xFF_u8, b'I', b'D']), ColumnFormat::Str);
    assert_eq!(INT_FIELDS.len(), 34);
}

#[test]
fn a_row_value_keeps_its_php_type() {
    // A non-string stays what it is, so `wpdb` formats it and skips its length check; an array goes through
    // `prepare()`'s `_doing_it_wrong( 'Unsupported value type' )`.
    let row = PostRow::new()
        .with("post_title", RowValue::Int(5))
        .with("post_excerpt", RowValue::Php(HookValue::Bool(true)))
        .with(
            "post_content",
            RowValue::Php(HookValue::Float(-1.2345678901234567e308)),
        )
        .with("menu_order", RowValue::Str(b"\n5".to_vec()))
        .with("guid", RowValue::Php(HookValue::Array(Vec::new())))
        .with([0xFF_u8, b'k'], RowValue::Null);
    assert_eq!(row.get("post_title"), Some(&RowValue::Int(5)));
    assert_eq!(
        row.get("post_excerpt"),
        Some(&RowValue::Php(HookValue::Bool(true)))
    );
    assert_eq!(
        row.get("guid"),
        Some(&RowValue::Php(HookValue::Array(Vec::new())))
    );
    assert_eq!(row.get([0xFF_u8, b'k']), Some(&RowValue::Null));
    assert_eq!(row.columns().last().unwrap().0, vec![0xFF, b'k']);
}

#[test]
fn a_write_outcome_carries_last_error() {
    let written: WriteOutcome<PostId> = WriteOutcome::Written(PostId(4));
    assert_eq!(written.last_error(), None);
    assert_eq!(written.written(), Some(PostId(4)));

    let failed: WriteOutcome<u64> =
        WriteOutcome::failed("Duplicate entry '100' for key 'wp_posts.PRIMARY'");
    assert_eq!(
        failed.last_error(),
        Some("Duplicate entry '100' for key 'wp_posts.PRIMARY'")
    );
    assert_eq!(failed.clone().written(), None);

    let repo: Rc<dyn PostRepository> = Rc::new(Dummy);
    let unknown = repo
        .insert(&PostRow::new().with("bogus", RowValue::Int(1)))
        .unwrap();
    assert!(unknown.last_error().is_some_and(|e| e.contains("bogus")));
    let empty = repo.update(PostId(3), &PostRow::new()).unwrap();
    assert!(empty.last_error().is_some());
}

struct Ports;

impl Translator for Ports {
    fn translate(&self, text: &str, _: Option<&str>) -> Result<String, PostError> {
        if text == "throw" {
            Err(HookError::message("gettext threw").into())
        } else {
            Ok(text.to_string())
        }
    }
}

impl Clock for Ports {
    fn now_local(&self) -> Result<String, PostError> {
        Err(HookError::message("pre_option_gmt_offset threw").into())
    }
    fn now_gmt(&self) -> String {
        "2024-01-01 00:00:00".to_string()
    }
    fn date_from_gmt(&self, gmt: &[u8]) -> Result<Vec<u8>, PostError> {
        Ok(gmt.to_vec())
    }
    fn gmt_from_date(&self, local: &[u8]) -> Result<Vec<u8>, PostError> {
        Ok(local.to_vec())
    }
    fn timestamp(&self, _: &[u8]) -> Option<i64> {
        Some(0)
    }
}

impl RewriteContext for Ports {
    fn rewrite_applies(&self) -> Result<bool, PostError> {
        Err(HookError::message("option_permalink_structure threw").into())
    }
}

#[test]
fn every_port_that_runs_a_callback_can_throw() {
    let translator: Rc<dyn Translator> = Rc::new(Ports);
    let clock: Rc<dyn Clock> = Rc::new(Ports);
    let rewrite: Rc<dyn RewriteContext> = Rc::new(Ports);
    assert_eq!(translator.translate("Posts", None).unwrap(), "Posts");
    assert!(matches!(
        translator.translate("throw", Some("ctx")),
        Err(PostError::Hook(_))
    ));
    assert!(matches!(clock.now_local(), Err(PostError::Hook(_))));
    assert_eq!(clock.now_gmt(), "2024-01-01 00:00:00");
    assert_eq!(clock.gmt_from_date(b"x").unwrap(), b"x".to_vec());
    assert!(matches!(rewrite.rewrite_applies(), Err(PostError::Hook(_))));
}

struct ThrowingSlugs;

impl BadSlugs for ThrowingSlugs {
    fn bad_attachment_slug(&self, _: &[u8]) -> Result<bool, PostError> {
        Ok(false)
    }
    fn bad_hierarchical_slug(&self, _: &[u8], _: &str, _: PostParent) -> Result<bool, PostError> {
        Ok(true)
    }
    fn bad_flat_slug(&self, _: &[u8], _: &str) -> Result<bool, PostError> {
        Err(HookError::message("thrown").into())
    }
}

#[test]
fn a_bad_slug_check_can_throw() {
    // A `wp_unique_post_slug_is_bad_*` callback's exception stops the use case before the slug check.
    let slugs: Rc<dyn BadSlugs> = Rc::new(ThrowingSlugs);
    assert!(!slugs.bad_attachment_slug(b"a").unwrap());
    assert!(
        slugs
            .bad_hierarchical_slug(b"a", "page", PostParent::Null)
            .unwrap()
    );
    assert!(matches!(
        slugs.bad_flat_slug(b"a", "post"),
        Err(PostError::Hook(_))
    ));
}

/// A PHP string a bridge passed through a hook untouched.
struct UnreadPhpString(Vec<u8>);

fn readable(any: &dyn std::any::Any) -> bool {
    any.is::<UnreadPhpString>()
}

fn read(any: &dyn std::any::Any) -> Option<HookValue> {
    any.downcast_ref::<UnreadPhpString>()
        .map(|s| HookValue::Bytes(s.0.clone()))
}

#[test]
fn a_hook_value_normalises_to_null_int_or_string() {
    // `Php` never holds what `Null`, `Int` or `Str` expresses, so every repository prints a PHP string the same way.
    speedyspec_hooks::set_opaque_reader(speedyspec_hooks::OpaqueReader { readable, read });
    assert_eq!(RowValue::from_hook(HookValue::Null), RowValue::Null);
    assert_eq!(RowValue::from_hook(HookValue::Int(-5)), RowValue::Int(-5));
    assert_eq!(
        RowValue::from_hook(HookValue::Str("publish".to_string())),
        RowValue::Str(b"publish".to_vec())
    );
    assert_eq!(
        RowValue::from_hook(HookValue::Bytes(vec![0xFF, b'x'])),
        RowValue::Str(vec![0xFF, b'x'])
    );
    let unread: Rc<dyn std::any::Any> = Rc::new(UnreadPhpString(b"draft".to_vec()));
    assert_eq!(
        RowValue::from_hook(HookValue::Opaque(unread)),
        RowValue::Str(b"draft".to_vec())
    );

    // A float, a bool, an array and an object the bridge keeps opaque stay PHP values.
    assert_eq!(
        RowValue::from_hook(HookValue::Float(1.5)),
        RowValue::Php(HookValue::Float(1.5))
    );
    assert_eq!(
        RowValue::from_hook(HookValue::Bool(false)),
        RowValue::Php(HookValue::Bool(false))
    );
    assert_eq!(
        RowValue::from_hook(HookValue::Array(Vec::new())),
        RowValue::Php(HookValue::Array(Vec::new()))
    );
    let object: Rc<dyn std::any::Any> = Rc::new(7_u8);
    let kept = RowValue::from_hook(HookValue::Opaque(object.clone()));
    assert!(matches!(kept, RowValue::Php(HookValue::Opaque(h)) if Rc::ptr_eq(&h, &object)));
}
