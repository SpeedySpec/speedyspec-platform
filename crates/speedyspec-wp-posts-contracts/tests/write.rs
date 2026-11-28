//! The write surface and the ports to unported code compile and stay object-safe.

use std::rc::Rc;

use speedyspec_php_serialize::PhpValue;
use speedyspec_wp_hooks_contracts::{HookError, HookValue};
use speedyspec_wp_posts_contracts::{
    Charset, Comments, Post, PostArg, PostError, PostId, PostObject, PostRef, PostReturn,
    PostStatusName, PostTypeName, PostWriteError, PostWrites, Revisions, Rewrite, Taxonomy, Theme,
    Uploads, Users, WriteResult,
};

struct Dummy;

impl PostWrites for Dummy {
    fn insert(&self, postarr: PhpValue, wp_error: bool, _: bool) -> Result<WriteResult, PostError> {
        Ok(match (postarr, wp_error) {
            (PhpValue::Array(items), _) if items.is_empty() => Err(PostWriteError::EmptyContent),
            _ => Ok(PostId(5)),
        })
    }
    fn update(&self, _: PhpValue, _: bool, _: bool) -> Result<WriteResult, PostError> {
        Ok(Err(PostWriteError::InvalidPost))
    }
    fn delete(&self, post_id: i64, _: bool) -> Result<PostReturn, PostError> {
        Ok(if post_id <= 0 {
            PostReturn::Value(PhpValue::Bool(false))
        } else {
            PostReturn::Post(Box::new(PostObject::new(Post {
                id: PostId(post_id as u64),
                ..Post::default()
            })))
        })
    }
    fn trash(&self, post: PostArg) -> Result<PostReturn, PostError> {
        // `wp_trash_post()` returns the object it was given.
        Ok(match post {
            PostArg::Object(object) => PostReturn::Post(object),
            PostArg::Ref(_) => PostReturn::Value(PhpValue::Null),
        })
    }
    fn untrash(&self, _: PostArg) -> Result<PostReturn, PostError> {
        Err(HookError::message("thrown").into())
    }
    fn publish(&self, _: PostArg) -> Result<(), PostError> {
        Ok(())
    }
    fn check_and_publish_future(&self, _: PostArg) -> Result<(), PostError> {
        Ok(())
    }
    fn transition(
        &self,
        _: &PostStatusName,
        _: &PostStatusName,
        post: &PostObject,
    ) -> Result<PostObject, PostError> {
        // As the binding does: the caller's handle when there is one, otherwise one object built for the call.
        let mut shared = post.clone();
        if shared.handle.is_none() {
            let built: Rc<dyn std::any::Any> = Rc::new(post.post.id);
            shared.handle = Some(HookValue::Opaque(built));
        }
        Ok(shared)
    }
    fn stick(&self, _: i64) -> Result<(), PostError> {
        Ok(())
    }
    fn unstick(&self, _: i64) -> Result<(), PostError> {
        Ok(())
    }
    fn add_ping(&self, _: PostArg, _: &PhpValue) -> Result<Option<u64>, PostError> {
        Ok(Some(1))
    }
    fn set_post_type(&self, _: i64, _: &PhpValue) -> Result<Option<u64>, PostError> {
        Ok(None)
    }
    fn delete_auto_drafts(&self) -> Result<(), PostError> {
        Ok(())
    }
}

impl Users for Dummy {
    fn is_user_logged_in(&self) -> Result<bool, PostError> {
        Ok(true)
    }
    fn get_current_user_id(&self) -> Result<u64, PostError> {
        Ok(1)
    }
    fn current_user_can(&self, _: &PhpValue, args: &[PhpValue]) -> Result<bool, PostError> {
        Ok(args.is_empty())
    }
}

impl Taxonomy for Dummy {
    fn update_object_term_cache(&self, _: &[PostId], _: &[PhpValue]) -> Result<(), PostError> {
        Ok(())
    }
    fn clean_object_term_cache(&self, _: &[PostId], _: &str) -> Result<(), PostError> {
        Ok(())
    }
    fn post_term_fields(&self, _: &PhpValue, _: &str, _: &str) -> Result<PhpValue, PostError> {
        Ok(PhpValue::Array(Vec::new()))
    }
    fn set_post_terms(
        &self,
        _: PostId,
        _: &PhpValue,
        _: &str,
        _: bool,
    ) -> Result<PhpValue, PostError> {
        Ok(PhpValue::Array(Vec::new()))
    }
    fn object_taxonomies(&self, _: &PostTypeName) -> Result<Vec<String>, PostError> {
        Ok(vec!["category".to_string(), "post_tag".to_string()])
    }
    fn default_terms(&self, _: &PostTypeName) -> Result<Vec<String>, PostError> {
        Ok(Vec::new())
    }
    fn assign_terms_cap(&self, taxonomy: &str) -> Result<Option<PhpValue>, PostError> {
        Ok((taxonomy == "category").then(|| PhpValue::Str(b"edit_posts".to_vec())))
    }
    fn get_object_terms(&self, _: PostId, _: &str, _: &PhpValue) -> Result<PhpValue, PostError> {
        Ok(PhpValue::Array(Vec::new()))
    }
    fn get_the_terms(&self, _: PostId, _: &str) -> Result<PhpValue, PostError> {
        Ok(PhpValue::Bool(false))
    }
    fn is_object_in_taxonomy(&self, _: &PostTypeName, taxonomy: &str) -> Result<bool, PostError> {
        Ok(taxonomy == "category")
    }
    fn delete_object_term_relationships(&self, _: PostId, _: &[String]) -> Result<(), PostError> {
        Ok(())
    }
}

impl Comments for Dummy {
    fn default_comment_status(
        &self,
        post_type: &PostTypeName,
        _: &str,
    ) -> Result<PhpValue, PostError> {
        let status: &[u8] = if post_type.as_str() == "page" {
            b"closed"
        } else {
            b"open"
        };
        Ok(PhpValue::Str(status.to_vec()))
    }
    fn trash_post_comments(&self, _: PostId) -> Result<(), PostError> {
        Ok(())
    }
    fn untrash_post_comments(&self, _: PostId) -> Result<(), PostError> {
        Ok(())
    }
    fn delete_post_comments(&self, _: PostId) -> Result<(), PostError> {
        Ok(())
    }
    fn defer_counting(&self, _: bool) -> Result<(), PostError> {
        Ok(())
    }
}

impl Rewrite for Dummy {
    fn permalink(&self, post: PostId) -> Result<Option<Vec<u8>>, PostError> {
        Ok((post.0 > 0).then(|| format!("http://example.org/?p={}", post.0).into_bytes()))
    }
}

impl Uploads for Dummy {
    fn insert_attachment(&self, _: PhpValue, _: bool) -> Result<PhpValue, PostError> {
        Ok(PhpValue::Int(0))
    }
    fn delete_attachment(&self, _: PostId, _: bool) -> Result<PhpValue, PostError> {
        Ok(PhpValue::Null)
    }
    fn update_attached_file(&self, _: PostId, _: &PhpValue) -> Result<(), PostError> {
        Ok(())
    }
    fn attachment_is(&self, kind: &str, _: PostId) -> Result<bool, PostError> {
        Ok(kind == "audio")
    }
    fn set_post_thumbnail(&self, _: PostId, _: i64) -> Result<(), PostError> {
        Ok(())
    }
    fn delete_post_thumbnail(&self, _: PostId) -> Result<(), PostError> {
        Ok(())
    }
}

impl Theme for Dummy {
    fn page_templates(&self, _: &Post, _: &PhpValue) -> Result<PhpValue, PostError> {
        Ok(PhpValue::Array(Vec::new()))
    }
    fn current_theme_supports(&self, feature: &str, _: &str) -> Result<bool, PostError> {
        Ok(feature == "post-thumbnails")
    }
}

impl Revisions for Dummy {
    fn save_post_revision(&self, _: PostId) -> Result<(), PostError> {
        Ok(())
    }
    fn delete_post_revision(&self, _: PostId) -> Result<(), PostError> {
        Ok(())
    }
}

impl Charset for Dummy {
    fn col_charset(&self, _: &str, column: &str) -> Result<Option<String>, PostError> {
        Ok((column != "ID").then(|| "utf8mb4".to_string()))
    }
}

#[test]
fn the_write_surface_is_object_safe_and_callable() {
    let writes: Rc<dyn PostWrites> = Rc::new(Dummy);

    assert_eq!(
        writes.insert(PhpValue::Null, true, true).unwrap(),
        Ok(PostId(5))
    );
    assert_eq!(
        writes
            .insert(PhpValue::Array(Vec::new()), false, true)
            .unwrap(),
        Err(PostWriteError::EmptyContent)
    );
    assert_eq!(
        writes.update(PhpValue::Null, true, false).unwrap(),
        Err(PostWriteError::InvalidPost)
    );
    assert_eq!(
        writes.delete(0, true).unwrap(),
        PostReturn::Value(PhpValue::Bool(false))
    );
    assert!(matches!(
        writes.delete(3, false).unwrap(),
        PostReturn::Post(post) if post.post.id == PostId(3) && post.handle.is_none()
    ));
    assert_eq!(
        writes.trash(PostRef::Current.into()).unwrap(),
        PostReturn::Value(PhpValue::Null)
    );
    let handle: Rc<dyn std::any::Any> = Rc::new(7_u64);
    let object = PostObject::with_handle(Post::default(), HookValue::Opaque(handle.clone()));
    let PostReturn::Post(trashed) = writes.trash(object.into()).unwrap() else {
        panic!("not the post");
    };
    assert!(matches!(&trashed.handle, Some(HookValue::Opaque(h)) if Rc::ptr_eq(h, &handle)));
    assert!(matches!(
        writes.untrash(PostRef::Id(PostId(1)).into()),
        Err(PostError::Hook(_))
    ));
    writes.publish(PostRef::Id(PostId(1)).into()).unwrap();
    writes
        .publish(PostObject::with_handle(Post::default(), HookValue::Int(1)).into())
        .unwrap();
    writes
        .check_and_publish_future(PostRef::Current.into())
        .unwrap();
    let built = writes
        .transition(
            &PostStatusName::new("publish"),
            &PostStatusName::new("new"),
            &PostObject::new(Post::default())
                .with_property("page_template", HookValue::Str("default".to_string())),
        )
        .unwrap();
    assert!(matches!(built.handle, Some(HookValue::Opaque(_))));
    assert_eq!(
        built.property("page_template"),
        Some(&HookValue::Str("default".to_string()))
    );
    let kept = writes
        .transition(
            &PostStatusName::new("draft"),
            &PostStatusName::new("publish"),
            &built,
        )
        .unwrap();
    assert_eq!(kept, built);
    writes.stick(-3).unwrap();
    writes.unstick(3).unwrap();
    assert_eq!(
        writes
            .add_ping(
                PostRef::Id(PostId(1)).into(),
                &PhpValue::Str(b"http://a.example/".to_vec())
            )
            .unwrap(),
        Some(1)
    );
    assert_eq!(
        writes
            .set_post_type(1, &PhpValue::Str(b"page".to_vec()))
            .unwrap(),
        None
    );
    writes.delete_auto_drafts().unwrap();
}

#[test]
fn the_ports_to_unported_code_are_object_safe_and_callable() {
    let users: Rc<dyn Users> = Rc::new(Dummy);
    let taxonomy: Rc<dyn Taxonomy> = Rc::new(Dummy);
    let comments: Rc<dyn Comments> = Rc::new(Dummy);
    let rewrite: Rc<dyn Rewrite> = Rc::new(Dummy);
    let uploads: Rc<dyn Uploads> = Rc::new(Dummy);
    let theme: Rc<dyn Theme> = Rc::new(Dummy);
    let revisions: Rc<dyn Revisions> = Rc::new(Dummy);
    let charset: Rc<dyn Charset> = Rc::new(Dummy);

    let cap = PhpValue::Str(b"publish_post".to_vec());
    assert!(users.current_user_can(&cap, &[]).unwrap());
    assert!(!users.current_user_can(&cap, &[PhpValue::Int(4)]).unwrap());
    assert_eq!(users.get_current_user_id().unwrap(), 1);

    let post = PostTypeName::new("post");
    assert_eq!(
        taxonomy.object_taxonomies(&post).unwrap(),
        vec!["category".to_string(), "post_tag".to_string()]
    );
    assert!(taxonomy.default_terms(&post).unwrap().is_empty());
    assert!(taxonomy.assign_terms_cap("category").unwrap().is_some());
    assert!(taxonomy.assign_terms_cap("nope").unwrap().is_none());
    assert!(taxonomy.is_object_in_taxonomy(&post, "category").unwrap());
    taxonomy
        .delete_object_term_relationships(PostId(1), &["category".to_string()])
        .unwrap();
    assert_eq!(
        taxonomy.get_the_terms(PostId(1), "post_tag").unwrap(),
        PhpValue::Bool(false)
    );

    assert_eq!(
        comments
            .default_comment_status(&PostTypeName::new("page"), "comment")
            .unwrap(),
        PhpValue::Str(b"closed".to_vec())
    );
    comments.defer_counting(true).unwrap();
    comments.delete_post_comments(PostId(1)).unwrap();
    comments.defer_counting(false).unwrap();

    assert_eq!(
        rewrite.permalink(PostId(7)).unwrap(),
        Some(b"http://example.org/?p=7".to_vec())
    );
    assert_eq!(rewrite.permalink(PostId(0)).unwrap(), None);

    assert_eq!(
        uploads.insert_attachment(PhpValue::Null, false).unwrap(),
        PhpValue::Int(0)
    );
    assert!(uploads.attachment_is("audio", PostId(2)).unwrap());
    uploads.set_post_thumbnail(PostId(1), 2).unwrap();

    assert!(
        theme
            .current_theme_supports("post-thumbnails", "post")
            .unwrap()
    );
    assert_eq!(
        theme
            .page_templates(&Post::default(), &PhpValue::Str(b"default".to_vec()))
            .unwrap(),
        PhpValue::Array(Vec::new())
    );

    revisions.delete_post_revision(PostId(9)).unwrap();
    assert_eq!(
        charset.col_charset("wp_posts", "post_title").unwrap(),
        Some("utf8mb4".to_string())
    );
}
