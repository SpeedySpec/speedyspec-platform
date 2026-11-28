//! The checks every post repository passes, so the memory and SQL repositories answer alike.

use crate::{
    CommentStatus, LastPostField, MimeFilter, MimeType, PageRow, PingStatus, Post, PostDate,
    PostDateTime, PostId, PostParent, PostRepository, PostRow, PostStatusName, PostTypeName,
    RowValue, Slug, Timezone,
};
use speedyspec_wp_hooks_contracts::HookValue;

fn ensure(check: &str, ok: bool, detail: impl FnOnce() -> String) {
    assert!(ok, "post repository check `{check}` failed: {}", detail());
}

fn slug(s: &str) -> Slug {
    Slug::from(s)
}

fn post_type(s: &str) -> PostTypeName {
    PostTypeName::new(s)
}

/// The row `compact()` builds for `post`, with `$data['ID']` set to `id`.
fn with_id(post: &Post, id: i64) -> PostRow {
    PostRow::from_post(post).with("ID", RowValue::Int(id))
}

/// An id as PHP's `(int)` holds it.
fn as_int(id: PostId) -> i64 {
    PostParent::from(id).prepared()
}

/// A post of `post_type` named `name` under `parent`, with an auto id.
fn named(name: &str, kind: &str, parent: i64) -> Post {
    Post {
        name: slug(name),
        post_type: post_type(kind),
        parent: PostParent::Int(parent),
        ..Post::default()
    }
}

/// Runs every check against `repo`: [`read_suite`], then [`write_suite`] on the same table. `repo` must start with an
/// empty table and a fresh auto-increment. Panics naming the failing check.
///
/// It is the suite for a repository that behaves as MySQL does under WordPress's `sql_mode`. An engine that does not
/// (SQLite stores a `-5` parent as `-5` and rejects a `NULL` one in an `UPDATE`) runs [`read_suite`] alone.
pub fn post_repository_suite(repo: &dyn PostRepository) {
    read_suite(repo);
    write_suite(repo);
}

/// The insert and read statements' checks. `repo` must start with an empty table and a fresh auto-increment
/// (`insert_with_id_is_honoured` expects 101 after an explicit 100). Panics naming the failing check.
///
/// The insert checks run first; the read statements' checks run next, on post types of their own (`rd-*`), and the
/// ones that read every page or every attachment compare what the table held before and after their inserts. Nothing
/// here depends on how the engine stores a value out of range or `NULL`, so every engine runs it.
pub fn read_suite(repo: &dyn PostRepository) {
    let insert = |post: &Post| {
        repo.insert(&PostRow::from_post(post))
            .expect("insert")
            .written()
            .expect("the insert returns an id")
    };
    let find = |id: PostId| repo.find(id).expect("find");

    // insert_returns_increasing_ids
    let a = insert(&named("first", "post", 0));
    let b = insert(&named("second", "post", 0));
    let c = insert(&named("third", "page", 0));
    ensure(
        "insert_returns_increasing_ids",
        PostId(0) < a && a < b && b < c,
        || format!("{a:?} {b:?} {c:?}"),
    );

    // insert_with_id_is_honoured: the auto-increment continues past an explicit ID, as MySQL's does; `post.id` is
    // not in the row.
    let explicit = repo
        .insert(&with_id(
            &Post {
                id: PostId(7_777),
                ..named("imported", "post", 0)
            },
            100,
        ))
        .expect("insert")
        .written()
        .expect("the insert returns an id");
    let next = insert(&named("after-import", "post", 0));
    ensure(
        "insert_with_id_is_honoured",
        explicit == PostId(100) && next == PostId(101),
        || format!("{explicit:?} {next:?}"),
    );

    // insert_taken_id_is_none
    let taken = repo
        .insert(&with_id(&named("duplicate", "post", 0), 100))
        .expect("insert")
        .written();
    let kept = find(PostId(100)).map(|p| p.name);
    ensure(
        "insert_taken_id_is_none",
        taken.is_none() && kept == Some(slug("imported")),
        || format!("{taken:?} {kept:?}"),
    );

    // find_returns_what_insert_wrote: every column, a zero date among them.
    let full = Post {
        id: PostId(0),
        author: 7,
        date: PostDate {
            local: PostDateTime::new("2024-01-05 10:20:30"),
            gmt: PostDateTime::ZERO,
        },
        content: b"<p>Hello, \xc3\xa9t\xc3\xa9</p>\x00tail".to_vec(),
        title: b"A title".to_vec(),
        excerpt: b"An excerpt".to_vec(),
        status: PostStatusName::new("draft"),
        comment_status: CommentStatus::new("closed"),
        ping_status: PingStatus::new("closed"),
        password: b"secret".to_vec(),
        name: slug("%e2%9c%93-full"),
        to_ping: b"http://a.example/".to_vec(),
        pinged: b"http://b.example/".to_vec(),
        modified: PostDate {
            local: PostDateTime::new("2024-02-06 11:21:31"),
            gmt: PostDateTime::new("2024-02-06 16:21:31"),
        },
        content_filtered: b"filtered".to_vec(),
        parent: b.into(),
        guid: b"http://example.org/?p=1".to_vec(),
        menu_order: -3,
        post_type: post_type("page"),
        mime_type: MimeType::from("image/png"),
        comment_count: 0,
    };
    let full_id = insert(&full);
    let read = find(full_id);
    let expected = Post {
        id: full_id,
        ..full.clone()
    };
    ensure(
        "find_returns_what_insert_wrote",
        read.as_ref() == Some(&expected),
        || format!("{read:?}"),
    );

    // insert_writes_comment_count_only_when_the_row_has_it: `compact()` leaves it out, a callback may add it.
    let counted = insert(&Post {
        comment_count: 5,
        ..named("counted", "post", 0)
    });
    let count = find(counted).map(|p| p.comment_count);
    let added = repo
        .insert(
            &PostRow::from_post(&named("counted-added", "post", 0))
                .with("comment_count", RowValue::Int(5)),
        )
        .expect("insert")
        .written()
        .expect("the insert returns an id");
    let added_count = find(added).map(|p| p.comment_count);
    ensure(
        "insert_writes_comment_count_only_when_the_row_has_it",
        count == Some(0) && added_count == Some(5),
        || format!("{count:?} {added_count:?}"),
    );

    // insert_leaves_a_dropped_column_to_its_default: a callback that unsets `post_status` and `post_content`.
    let mut dropped = PostRow::from_post(&Post {
        status: PostStatusName::new("draft"),
        content: b"gone".to_vec(),
        ..named("dropped", "post", 0)
    });
    dropped.remove("post_status");
    dropped.remove("post_content");
    let dropped_id = repo
        .insert(&dropped)
        .expect("insert")
        .written()
        .expect("the insert returns an id");
    let defaulted = find(dropped_id);
    ensure(
        "insert_leaves_a_dropped_column_to_its_default",
        defaulted.as_ref().is_some_and(|p| {
            p.status == PostStatusName::new("publish")
                && p.content.is_empty()
                && p.name == slug("dropped")
        }),
        || format!("{defaulted:?}"),
    );

    // insert_of_an_added_unknown_column_or_an_empty_row_is_none: MySQL rejects both, so `$wpdb->insert` is false.
    let unknown = repo
        .insert(
            &PostRow::from_post(&named("unknown-column", "post", 0))
                .with("bogus", RowValue::Str(b"x".to_vec())),
        )
        .expect("insert")
        .written();
    let unknown_owner = repo
        .flat_slug_owner(&slug("unknown-column"), &post_type("post"), PostId(0))
        .expect("flat_slug_owner");
    let empty = repo.insert(&PostRow::new()).expect("insert").written();
    ensure(
        "insert_of_an_added_unknown_column_or_an_empty_row_is_none",
        unknown.is_none() && unknown_owner.is_none() && empty.is_none(),
        || format!("{unknown:?} {unknown_owner:?} {empty:?}"),
    );

    // insert_formats_each_php_type: an int under `%s` is its decimal text, a bool `'1'` or `''`, a float its
    // `(string)`, and a string under `%d` is read as `sprintf( '%d' )` reads it.
    let typed = repo
        .insert(
            &PostRow::from_post(&named("typed", "post", 0))
                .with("post_title", RowValue::Int(5))
                .with("post_excerpt", RowValue::Php(HookValue::Bool(true)))
                .with("post_password", RowValue::Php(HookValue::Bool(false)))
                .with("post_content", RowValue::Php(HookValue::Float(1.5)))
                .with("menu_order", RowValue::Str(b"4x".to_vec()))
                .with("post_author", RowValue::Php(HookValue::Float(2.9))),
        )
        .expect("insert")
        .written()
        .expect("the insert returns an id");
    let typed = find(typed);
    ensure(
        "insert_formats_each_php_type",
        typed.as_ref().is_some_and(|p| {
            p.title == b"5".to_vec()
                && p.excerpt == b"1".to_vec()
                && p.password.is_empty()
                && p.content == b"1.5".to_vec()
                && p.menu_order == 4
                && p.author == 2
        }),
        || format!("{typed:?}"),
    );

    // a_failed_insert_carries_last_error: the engine's message comes back with the failure.
    let failed = repo
        .insert(&PostRow::from_post(&named("failed", "post", 0)).with("bogus", RowValue::Int(1)))
        .expect("insert");
    ensure(
        "a_failed_insert_carries_last_error",
        failed.last_error().is_some_and(|e| !e.is_empty()),
        || format!("{failed:?}"),
    );

    // find_missing_is_none
    let missing = find(PostId(9_999));
    let zero = find(PostId(0));
    ensure(
        "find_missing_is_none",
        missing.is_none() && zero.is_none(),
        || format!("{missing:?} {zero:?}"),
    );

    // attachment_owner_spans_types_and_skips_except
    let shared_post = insert(&named("shared", "post", 0));
    insert(&named("shared", "page", 4));
    let any = repo
        .attachment_slug_owner(&slug("shared"), PostId(0))
        .expect("attachment_slug_owner");
    let past_one = repo
        .attachment_slug_owner(&slug("shared"), shared_post)
        .expect("attachment_slug_owner");
    let nobody = repo
        .attachment_slug_owner(&slug("unshared"), PostId(0))
        .expect("attachment_slug_owner");
    ensure(
        "attachment_owner_spans_types",
        any == Some(slug("shared")) && past_one == Some(slug("shared")) && nobody.is_none(),
        || format!("{any:?} {past_one:?} {nobody:?}"),
    );
    let only = insert(&named("lonely", "book", 0));
    let only_except = repo
        .attachment_slug_owner(&slug("lonely"), only)
        .expect("attachment_slug_owner");
    ensure(
        "attachment_owner_skips_except",
        only_except.is_none(),
        || format!("{only_except:?}"),
    );

    // hierarchical_owner_matches_type_or_attachment_under_the_parent
    let hier = |name: &str, kind: &str, except: PostId, parent: i64| {
        repo.hierarchical_slug_owner(
            &slug(name),
            &post_type(kind),
            except,
            PostParent::Int(parent),
        )
        .expect("hierarchical_slug_owner")
    };
    let chapter = insert(&named("chapter", "page", 50));
    insert(&named("image", "attachment", 50));
    insert(&named("note", "post", 50));
    let same = hier("chapter", "page", PostId(0), 50);
    let attachment = hier("image", "page", PostId(0), 50);
    let other_type = hier("note", "page", PostId(0), 50);
    let other_parent = hier("chapter", "page", PostId(0), 51);
    let excepted = hier("chapter", "page", chapter, 50);
    ensure(
        "hierarchical_owner_matches_type_or_attachment_under_the_parent",
        same == Some(slug("chapter"))
            && attachment == Some(slug("image"))
            && other_type.is_none()
            && other_parent.is_none()
            && excepted.is_none(),
        || format!("{same:?} {attachment:?} {other_type:?} {other_parent:?} {excepted:?}"),
    );

    // flat_owner_matches_the_type_only
    let flat = |name: &str, kind: &str, except: PostId| {
        repo.flat_slug_owner(&slug(name), &post_type(kind), except)
            .expect("flat_slug_owner")
    };
    let story = insert(&named("story", "book", 60));
    insert(&named("cover", "attachment", 0));
    let same_type = flat("story", "book", PostId(0));
    let wrong_type = flat("story", "post", PostId(0));
    let not_attachment = flat("cover", "book", PostId(0));
    let skipped = flat("story", "book", story);
    insert(&named("story", "magazine", 60));
    let other_type_same_parent = flat("story", "book", story);
    ensure(
        "flat_owner_matches_the_type_only",
        same_type == Some(slug("story"))
            && wrong_type.is_none()
            && not_attachment.is_none()
            && skipped.is_none()
            && other_type_same_parent.is_none(),
        || {
            format!(
                "{same_type:?} {wrong_type:?} {not_attachment:?} {skipped:?} {other_type_same_parent:?}"
            )
        },
    );

    // owner_returns_the_stored_name: PHP truth is the rule's to test, so "0" and "" come back as stored.
    insert(&named("0", "post", 0));
    insert(&named("", "post", 0));
    let zero_name = repo
        .flat_slug_owner(&slug("0"), &post_type("post"), PostId(0))
        .expect("flat_slug_owner");
    let empty_name = repo
        .flat_slug_owner(&slug(""), &post_type("post"), PostId(0))
        .expect("flat_slug_owner");
    let zero_any = repo
        .attachment_slug_owner(&slug("0"), PostId(0))
        .expect("attachment_slug_owner");
    let empty_hier = repo
        .hierarchical_slug_owner(&slug(""), &post_type("post"), PostId(0), PostParent::NONE)
        .expect("hierarchical_slug_owner");
    ensure(
        "owner_returns_the_stored_name",
        zero_name == Some(slug("0"))
            && empty_name == Some(slug(""))
            && zero_any == Some(slug("0"))
            && empty_hier == Some(slug("")),
        || format!("{zero_name:?} {empty_name:?} {zero_any:?} {empty_hier:?}"),
    );

    read_statements(repo);
}

/// The write statements' checks, on post types of their own (`wr-*`). Panics naming the failing check. It runs on an
/// empty table or after [`read_suite`]; the auto-draft check compares the ids before and after its inserts.
///
/// It pins MySQL's behaviour under WordPress's `sql_mode` (`wpdb::set_sql_mode()` removes the strict modes): a
/// negative parent or import id is clamped (the parent stored as `0`, the id taking the next auto-increment value), a
/// `NULL` parent is rejected in an `INSERT` (error 1048) and stored as `0` in an `UPDATE`, and an update counts the
/// rows it changed. A repository on an engine that does not behave that way (SQLite) runs [`read_suite`] alone, and
/// a SQL repository on MySQL must run with that `sql_mode` to pass.
pub fn write_suite(repo: &dyn PostRepository) {
    write_statements(repo);
}

/// Checks for repositories whose columns are binary-safe (memory, latin1 or binary tables); `post_repository_suite`
/// leaves them out. On stock utf8mb4 tables `wpdb::process_fields()` strips invalid text and `$wpdb->insert` returns
/// false, so a faithful SQL repository answers `Ok(None)` from `insert` and fails this suite by design. Like
/// `post_repository_suite`, it needs an empty table.
pub fn binary_safe_suite(repo: &dyn PostRepository) {
    // content_and_name_keep_invalid_utf8
    let raw_name = Slug::new(vec![b's', 0x00, 0xC2, 0xA0, 0xFF]);
    let raw_content = vec![0x00, 0xFF, 0xFE, b'c', 0xC3];
    let post = Post {
        name: raw_name.clone(),
        content: raw_content.clone(),
        ..Post::default()
    };
    let id = repo
        .insert(&PostRow::from_post(&post))
        .expect("insert")
        .written()
        .expect("the insert returns an id");
    let read = repo.find(id).expect("find");
    let owner = repo
        .flat_slug_owner(&raw_name, &PostTypeName::new("post"), PostId(0))
        .expect("flat_slug_owner");
    ensure(
        "content_and_name_keep_invalid_utf8",
        read == Some(Post { id, ..post }) && owner == Some(raw_name.clone()),
        || format!("{read:?} {owner:?}"),
    );
}

/// A post of `kind` named `name` with `status`.
fn with_status(name: &str, kind: &str, status: &str) -> Post {
    Post {
        status: PostStatusName::new(status),
        ..named(name, kind, 0)
    }
}

/// Sorted, for statements with no `ORDER BY`: their order is the engine's.
fn sorted<T: Ord>(mut rows: Vec<T>) -> Vec<T> {
    rows.sort();
    rows
}

/// What `count_by_status` answers, by status name, sorted.
fn counts(rows: Vec<(PostStatusName, u64)>) -> Vec<(String, u64)> {
    sorted(
        rows.into_iter()
            .map(|(status, n)| (status.as_str().to_string(), n))
            .collect(),
    )
}

/// What `count_attachments` answers: the mime counts by type, sorted, and the trash count.
fn attachment_counts(
    repo: &dyn PostRepository,
    mime_type: Option<&MimeFilter>,
) -> (Vec<(Vec<u8>, u64)>, u64) {
    let (rows, trash) = repo
        .count_attachments(mime_type)
        .expect("count_attachments");
    let rows = rows
        .into_iter()
        .map(|(mime, n)| (mime.as_bytes().to_vec(), n))
        .collect();
    (sorted(rows), trash)
}

/// `after` less `before`, by mime type, leaving out the types whose count did not change.
fn grown(before: &[(Vec<u8>, u64)], after: &[(Vec<u8>, u64)]) -> Vec<(String, u64)> {
    after
        .iter()
        .filter_map(|(mime, n)| {
            let was = before
                .iter()
                .find(|(m, _)| m == mime)
                .map_or(0, |(_, n)| *n);
            (*n != was).then(|| (String::from_utf8_lossy(mime).into_owned(), n - was))
        })
        .collect()
}

fn read_statements(repo: &dyn PostRepository) {
    let insert = |post: &Post| {
        repo.insert(&PostRow::from_post(post))
            .expect("insert")
            .written()
            .expect("the insert returns an id")
    };
    let types = |names: &[&str]| names.iter().map(|n| post_type(n)).collect::<Vec<_>>();

    // find_many_reads_every_found_id: rows for the ids that exist, once each, in the engine's order.
    let zero_dated = insert(&Post {
        date: PostDate {
            local: PostDateTime::ZERO,
            gmt: PostDateTime::ZERO,
        },
        modified: PostDate {
            local: PostDateTime::ZERO,
            gmt: PostDateTime::ZERO,
        },
        ..named("zero-dated", "rd-many", 0)
    });
    let child = insert(&named("child", "rd-many", as_int(zero_dated)));
    let mut many = repo
        .find_many(&[child, PostId(9_998), zero_dated, child])
        .expect("find_many");
    many.sort_by_key(|p| p.id);
    let expected = vec![
        Post {
            id: zero_dated,
            ..named("zero-dated", "rd-many", 0)
        },
        Post {
            id: child,
            ..named("child", "rd-many", as_int(zero_dated))
        },
    ];
    ensure(
        "find_many_reads_every_found_id",
        many == expected && many[0].date.local.is_zero() && many[0].modified.gmt.is_zero(),
        || format!("{many:?}"),
    );

    // find_many_and_parent_ids_of_nothing_are_empty
    let none = repo.find_many(&[]).expect("find_many");
    let no_parents = repo.parent_ids(&[]).expect("parent_ids");
    let missing = repo.parent_ids(&[PostId(9_998)]).expect("parent_ids");
    ensure(
        "find_many_and_parent_ids_of_nothing_are_empty",
        none.is_empty() && no_parents.is_empty() && missing.is_empty(),
        || format!("{none:?} {no_parents:?} {missing:?}"),
    );

    // parent_ids_pairs_each_id_with_its_parent
    let parents = sorted(
        repo.parent_ids(&[child, zero_dated, PostId(9_998), child])
            .expect("parent_ids"),
    );
    ensure(
        "parent_ids_pairs_each_id_with_its_parent",
        parents == vec![(zero_dated, PostId(0)), (child, zero_dated)],
        || format!("{parents:?}"),
    );

    // pages_by_names_orders_published_first_then_drafts_last
    let draft = insert(&with_status("about", "rd-page", "draft"));
    let private = insert(&with_status("about", "rd-page", "private"));
    let published = insert(&with_status("about", "rd-page", "publish"));
    let pending = insert(&with_status("about", "rd-page", "pending"));
    let republished = insert(&with_status("about", "rd-page", "publish"));
    let team = insert(&Post {
        parent: published.into(),
        ..with_status("team", "rd-page", "publish")
    });
    let auto_draft = insert(&with_status("team", "rd-page", "auto-draft"));
    let photo = insert(&with_status("about", "rd-attach", "inherit"));
    insert(&with_status("about", "rd-other", "publish"));
    let pages = repo
        .pages_by_names(&[slug("about"), slug("team")], &types(&["rd-page"]))
        .expect("pages_by_names");
    let order: Vec<PostId> = pages.iter().map(|p| p.id).collect();
    let expected = vec![
        published,
        republished,
        team,
        private,
        draft,
        pending,
        auto_draft,
    ];
    let team_row = pages.iter().find(|p| p.id == team);
    ensure(
        "pages_by_names_orders_published_first_then_drafts_last",
        order == expected
            && team_row
                == Some(&PageRow {
                    id: team,
                    name: slug("team"),
                    parent: published,
                    post_type: post_type("rd-page"),
                }),
        || format!("{pages:?}"),
    );

    // pages_by_names_matches_any_listed_type
    let both = repo
        .pages_by_names(&[slug("about")], &types(&["rd-page", "rd-attach"]))
        .expect("pages_by_names");
    let both: Vec<PostId> = both.iter().map(|p| p.id).collect();
    ensure(
        "pages_by_names_matches_any_listed_type",
        both == vec![published, republished, private, photo, draft, pending],
        || format!("{both:?}"),
    );

    // pages_by_names_of_nothing_is_empty
    let no_names = repo
        .pages_by_names(&[], &types(&["rd-page"]))
        .expect("pages_by_names");
    let no_types = repo
        .pages_by_names(&[slug("about")], &[])
        .expect("pages_by_names");
    let unknown = repo
        .pages_by_names(&[slug("nowhere")], &types(&["rd-page"]))
        .expect("pages_by_names");
    ensure(
        "pages_by_names_of_nothing_is_empty",
        no_names.is_empty() && no_types.is_empty() && unknown.is_empty(),
        || format!("{no_names:?} {no_types:?} {unknown:?}"),
    );

    // all_page_ids_lists_every_page: whatever the status, and nothing of another type.
    let before = sorted(repo.all_page_ids().expect("all_page_ids"));
    let page_a = insert(&with_status("rd-a", "page", "draft"));
    let page_b = insert(&with_status("rd-b", "page", "publish"));
    insert(&with_status("rd-c", "rd-page", "publish"));
    let after = sorted(repo.all_page_ids().expect("all_page_ids"));
    let mut expected = before.clone();
    expected.extend([page_a, page_b]);
    let expected = sorted(expected);
    ensure("all_page_ids_lists_every_page", after == expected, || {
        format!("{before:?} {after:?}")
    });

    // count_by_status_groups_by_status
    for (status, author) in [
        ("publish", 7),
        ("publish", 8),
        ("draft", 7),
        ("private", 7),
        ("private", 8),
        ("private", 8),
    ] {
        insert(&Post {
            author,
            ..with_status("counted", "rd-count", status)
        });
    }
    let all = counts(
        repo.count_by_status(&post_type("rd-count"), None)
            .expect("count_by_status"),
    );
    let own = counts(
        repo.count_by_status(&post_type("rd-count"), Some(7))
            .expect("count_by_status"),
    );
    let none_own = counts(
        repo.count_by_status(&post_type("rd-count"), Some(9))
            .expect("count_by_status"),
    );
    let empty_type = repo
        .count_by_status(&post_type("rd-none"), None)
        .expect("count_by_status");
    let entry = |s: &str, n: u64| (s.to_string(), n);
    ensure(
        "count_by_status_groups_by_status",
        all == vec![entry("draft", 1), entry("private", 3), entry("publish", 2)]
            && own == vec![entry("draft", 1), entry("private", 1), entry("publish", 2)]
            && none_own == vec![entry("draft", 1), entry("publish", 2)]
            && empty_type.is_empty(),
        || format!("{all:?} {own:?} {none_own:?} {empty_type:?}"),
    );

    // count_attachments_counts_by_mime_type_and_trash: `mime_type` filters both statements, by
    // `wp_post_mime_type_where()`'s rule.
    let one = |s: &str| Some(MimeFilter::One(s.to_string()));
    let many = |items: &[&str]| {
        Some(MimeFilter::Many(
            items.iter().map(|i| i.to_string()).collect(),
        ))
    };
    let filters = [
        None,
        one("image"),
        one("image/png"),
        one("image/png, text"),
        // A string drops its empty items; an empty string has none left, so everything matches.
        one(",image/png,,"),
        one(""),
        // An array keeps its items: an empty one matches everything, even after a clause.
        many(&["image/png", ""]),
        many(&["image/png", "%/%"]),
        // `*` is not a wildcard item: it becomes `LIKE '%'`, which also matches everything.
        one("*"),
        // An array item holding a comma is one pattern: `= 'image/pngtext'`.
        many(&["image/png, text"]),
    ];
    let before: Vec<_> = filters
        .iter()
        .map(|f| attachment_counts(repo, f.as_ref()))
        .collect();
    for (mime, status) in [
        ("image/png", "inherit"),
        ("image/png", "publish"),
        ("image/jpeg", "inherit"),
        ("text/plain", "trash"),
        ("image/png", "trash"),
    ] {
        insert(&Post {
            mime_type: MimeType::from(mime),
            ..with_status("upload", "attachment", status)
        });
    }
    let after: Vec<_> = filters
        .iter()
        .map(|f| attachment_counts(repo, f.as_ref()))
        .collect();
    let delta: Vec<(Vec<(String, u64)>, u64)> = before
        .iter()
        .zip(&after)
        .map(|((b, bt), (a, at))| (grown(b, a), at - bt))
        .collect();
    let mime = |m: &str, n: u64| (m.to_string(), n);
    let every = || (vec![mime("image/jpeg", 1), mime("image/png", 2)], 2);
    let expected = vec![
        every(),
        (vec![mime("image/jpeg", 1), mime("image/png", 2)], 1),
        (vec![mime("image/png", 2)], 1),
        (vec![mime("image/png", 2)], 2),
        (vec![mime("image/png", 2)], 1),
        every(),
        every(),
        every(),
        every(),
        (vec![], 0),
    ];
    ensure(
        "count_attachments_counts_by_mime_type_and_trash",
        delta == expected,
        || format!("{delta:?}"),
    );

    // available_mime_types_lists_each_non_empty_type_once
    for mime in ["audio/ogg", "audio/ogg", "video/mp4", ""] {
        insert(&Post {
            mime_type: MimeType::from(mime),
            ..named("media", "rd-media", 0)
        });
    }
    insert(&Post {
        mime_type: MimeType::from("image/gif"),
        ..named("media", "rd-other-media", 0)
    });
    let available = sorted(
        repo.available_mime_types(&post_type("rd-media"))
            .expect("available_mime_types"),
    );
    let unknown = repo
        .available_mime_types(&post_type("rd-none"))
        .expect("available_mime_types");
    ensure(
        "available_mime_types_lists_each_non_empty_type_once",
        available == vec![MimeType::from("audio/ogg"), MimeType::from("video/mp4")]
            && unknown.is_empty(),
        || format!("{available:?} {unknown:?}"),
    );

    last_post_time(repo);
}

fn last_post_time(repo: &dyn PostRepository) {
    let insert = |post: &Post| {
        repo.insert(&PostRow::from_post(post))
            .expect("insert")
            .written()
            .expect("the insert returns an id")
    };
    let dated = |kind: &str, status: &str, local: &str, gmt: &str| Post {
        date: PostDate {
            local: PostDateTime::new(local),
            gmt: PostDateTime::new(gmt),
        },
        modified: PostDate {
            local: PostDateTime::new(local),
            gmt: PostDateTime::new(gmt),
        },
        ..with_status("dated", kind, status)
    };
    let last = |field: LastPostField, kinds: &[&str], timezone: Timezone| {
        let kinds: Vec<_> = kinds.iter().map(|k| post_type(k)).collect();
        repo.last_post_time(field, &kinds, timezone)
            .expect("last_post_time")
            .map(|d| d.as_str().to_string())
    };
    let at = |s: &str| Some(s.to_string());

    // last_post_time_orders_by_the_gmt_column: the newest published post by GMT, whichever column is read. Server
    // time is MySQL's `DATE_ADD(.., INTERVAL '0' SECOND)`: the string interval makes the result carry microseconds,
    // `.000000`, and WordPress returns it as is.
    insert(&dated(
        "rd-last",
        "publish",
        "2024-03-01 09:00:00",
        "2024-03-01 15:00:00",
    ));
    insert(&dated(
        "rd-last",
        "publish",
        "2024-03-02 01:00:00",
        "2024-03-01 20:00:00",
    ));
    insert(&dated(
        "rd-last",
        "draft",
        "2025-01-01 00:00:00",
        "2025-01-01 00:00:00",
    ));
    insert(&dated(
        "rd-last-b",
        "publish",
        "2024-02-01 00:00:00",
        "2024-02-01 00:00:00",
    ));
    let blog = last(LastPostField::Date, &["rd-last"], Timezone::Blog);
    let gmt = last(LastPostField::Modified, &["rd-last"], Timezone::Gmt);
    let server = last(LastPostField::Date, &["rd-last"], Timezone::Server);
    let either = last(
        LastPostField::Date,
        &["rd-last-b", "rd-last"],
        Timezone::Gmt,
    );
    ensure(
        "last_post_time_orders_by_the_gmt_column",
        blog == at("2024-03-02 01:00:00")
            && gmt == at("2024-03-01 20:00:00")
            && server == at("2024-03-01 20:00:00.000000")
            && either == at("2024-03-01 20:00:00"),
        || format!("{blog:?} {gmt:?} {server:?} {either:?}"),
    );

    // last_post_time_reads_zero_dates_back: a zero local date is a date, not "none".
    insert(&dated(
        "rd-last-zero",
        "publish",
        PostDateTime::ZERO.as_str(),
        "2024-01-01 00:00:00",
    ));
    let zero_local = last(LastPostField::Date, &["rd-last-zero"], Timezone::Blog);
    insert(&dated(
        "rd-last-zero-gmt",
        "publish",
        "2024-01-01 00:00:00",
        PostDateTime::ZERO.as_str(),
    ));
    let zero_gmt = last(
        LastPostField::Modified,
        &["rd-last-zero-gmt"],
        Timezone::Gmt,
    );
    ensure(
        "last_post_time_reads_zero_dates_back",
        zero_local == at(PostDateTime::ZERO.as_str())
            && zero_gmt == at(PostDateTime::ZERO.as_str()),
        || format!("{zero_local:?} {zero_gmt:?}"),
    );

    // last_post_time_of_nothing_is_none: no published post, an unknown type, or no type at all.
    insert(&dated(
        "rd-last-draft",
        "draft",
        "2024-01-01 00:00:00",
        "2024-01-01 00:00:00",
    ));
    let drafts = last(LastPostField::Date, &["rd-last-draft"], Timezone::Blog);
    let unknown = last(LastPostField::Date, &["rd-none"], Timezone::Gmt);
    let no_types = last(LastPostField::Modified, &[], Timezone::Blog);
    ensure(
        "last_post_time_of_nothing_is_none",
        drafts.is_none() && unknown.is_none() && no_types.is_none(),
        || format!("{drafts:?} {unknown:?} {no_types:?}"),
    );
}

fn write_statements(repo: &dyn PostRepository) {
    let insert = |post: &Post| {
        repo.insert(&PostRow::from_post(post))
            .expect("insert")
            .written()
            .expect("the insert returns an id")
    };
    let find = |id: PostId| repo.find(id).expect("find").expect("the post exists");
    let missing = PostId(9_996);

    // insert_negative_parent_is_stored_as_zero: WordPress sends `post_parent = -5`; MySQL stores 0 under its
    // `sql_mode`, and that is what is read back.
    let negative = insert(&named("wr-negative", "wr-page", -5));
    let stored = find(negative).parent;
    ensure(
        "insert_negative_parent_is_stored_as_zero",
        stored == PostParent::NONE,
        || format!("{stored:?}"),
    );

    // hierarchical_owner_under_a_negative_parent_matches_nothing: `post_parent = -5` matches no unsigned value, so
    // the top-level page does not hold the slug for a page under -5.
    let owner = |parent: PostParent| {
        repo.hierarchical_slug_owner(&slug("wr-clash"), &post_type("wr-page"), PostId(0), parent)
            .expect("hierarchical_slug_owner")
    };
    insert(&named("wr-clash", "wr-page", 0));
    let under_negative = owner(PostParent::Int(-5));
    let top_level = owner(PostParent::NONE);
    ensure(
        "hierarchical_owner_under_a_negative_parent_matches_nothing",
        under_negative.is_none() && top_level == Some(slug("wr-clash")),
        || format!("{under_negative:?} {top_level:?}"),
    );

    // import_id_taken_sees_existing_ids
    let taken = repo
        .import_id_taken(as_int(negative))
        .expect("import_id_taken");
    let free = repo
        .import_id_taken(as_int(missing))
        .expect("import_id_taken");
    let zero = repo.import_id_taken(0).expect("import_id_taken")
        || repo.import_id_taken(-5).expect("import_id_taken");
    ensure(
        "import_id_taken_sees_existing_ids",
        taken && !free && !zero,
        || format!("{taken} {free} {zero}"),
    );

    // insert_with_a_zero_or_negative_id_takes_the_next_id: `'abc'` casts to 0 and `-5` stays -5; both are sent as
    // the `ID` and MySQL gives the row the next auto-increment id (the unsigned column clamps -5 to 0).
    let auto = repo
        .insert(&PostRow::from_post(&named("wr-auto", "wr-post", 0)))
        .expect("insert")
        .written()
        .expect("the insert returns an id");
    let at_zero = repo
        .insert(&with_id(&named("wr-import-zero", "wr-post", 0), 0))
        .expect("insert")
        .written();
    let at_negative = repo
        .insert(&with_id(&named("wr-import-negative", "wr-post", 0), -5))
        .expect("insert")
        .written();
    let names = |id: Option<PostId>| {
        id.and_then(|id| repo.find(id).expect("find"))
            .map(|p| p.name)
    };
    ensure(
        "insert_with_a_zero_or_negative_id_takes_the_next_id",
        at_zero == Some(PostId(auto.0 + 1))
            && at_negative == Some(PostId(auto.0 + 2))
            && names(at_zero) == Some(slug("wr-import-zero"))
            && names(at_negative) == Some(slug("wr-import-negative")),
        || format!("{auto:?} {at_zero:?} {at_negative:?}"),
    );

    // insert_with_a_null_id_takes_the_next_id: `ID` is AUTO_INCREMENT, so a callback's `'ID' => null` is not rejected.
    let after_negative = at_negative.expect("the negative id insert");
    let null_id = repo
        .insert(&PostRow::from_post(&named("wr-null-id", "wr-post", 0)).with("ID", RowValue::Null))
        .expect("insert")
        .written();
    ensure(
        "insert_with_a_null_id_takes_the_next_id",
        null_id == Some(PostId(after_negative.0 + 1)),
        || format!("{null_id:?}"),
    );

    // insert_with_id_before_the_other_columns_is_honoured: `import_id` replacing a callback's `ID` where it stands.
    let first_id = PostRow::new()
        .with("ID", RowValue::Int(8_700))
        .with("post_name", RowValue::Str(b"wr-id-first".to_vec()))
        .with("post_type", RowValue::Str(b"wr-post".to_vec()));
    let placed = repo.insert(&first_id).expect("insert").written();
    ensure(
        "insert_with_id_before_the_other_columns_is_honoured",
        placed == Some(PostId(8_700)) && names(placed) == Some(slug("wr-id-first")),
        || format!("{placed:?}"),
    );

    // insert_null_parent_writes_no_row: `NULL` in the `NOT NULL` column is error 1048, so `$wpdb->insert` is false.
    let null_parent = repo
        .insert(&PostRow::from_post(&Post {
            parent: PostParent::Null,
            ..named("wr-null-parent", "wr-page", 0)
        }))
        .expect("insert")
        .written();
    let owner_of_null = repo
        .flat_slug_owner(&slug("wr-null-parent"), &post_type("wr-page"), PostId(0))
        .expect("flat_slug_owner");
    ensure(
        "insert_null_parent_writes_no_row",
        null_parent.is_none() && owner_of_null.is_none(),
        || format!("{null_parent:?} {owner_of_null:?}"),
    );

    // hierarchical_owner_under_a_null_parent_is_the_top_level_one: `%d` prints `null` as 0.
    let under_null = owner(PostParent::Null);
    ensure(
        "hierarchical_owner_under_a_null_parent_is_the_top_level_one",
        under_null == Some(slug("wr-clash")),
        || format!("{under_null:?}"),
    );

    // update_writes_every_column_but_id_and_comment_count: the rows changed, as MySQL counts them.
    let original = Post {
        comment_count: 0,
        ..named("wr-before", "wr-post", 0)
    };
    let id = insert(&original);
    let changed = Post {
        id,
        author: 9,
        date: PostDate {
            local: PostDateTime::new("2024-04-01 08:00:00"),
            gmt: PostDateTime::new("2024-04-01 12:00:00"),
        },
        content: b"new content".to_vec(),
        title: b"New title".to_vec(),
        excerpt: b"New excerpt".to_vec(),
        status: PostStatusName::new("pending"),
        comment_status: CommentStatus::new("closed"),
        ping_status: PingStatus::new("closed"),
        password: b"pw".to_vec(),
        name: slug("wr-after"),
        to_ping: b"http://c.example/".to_vec(),
        pinged: b"http://d.example/".to_vec(),
        modified: PostDate {
            local: PostDateTime::new("2024-04-02 08:00:00"),
            gmt: PostDateTime::ZERO,
        },
        content_filtered: b"cf".to_vec(),
        parent: PostParent::Int(3),
        guid: b"http://example.org/?p=wr".to_vec(),
        menu_order: 4,
        post_type: post_type("wr-other"),
        mime_type: MimeType::from("text/plain"),
        comment_count: 7,
    };
    let changed_row = PostRow::from_post(&changed);
    let first = repo.update(id, &changed_row).expect("update").written();
    let read = find(id);
    let again = repo.update(id, &changed_row).expect("update").written();
    let nobody = repo
        .update(missing, &changed_row)
        .expect("update")
        .written();
    ensure(
        "update_writes_every_column_but_id_and_comment_count",
        first == Some(1)
            && read
                == Post {
                    comment_count: 0,
                    ..changed.clone()
                }
            && again == Some(0)
            && nobody == Some(0)
            && repo.find(missing).expect("find").is_none(),
        || format!("{first:?} {read:?} {again:?} {nobody:?}"),
    );

    // update_negative_parent_is_stored_as_zero
    repo.update(
        id,
        &PostRow::from_post(&Post {
            parent: PostParent::Int(-7),
            ..changed.clone()
        }),
    )
    .expect("update")
    .written();
    let stored = find(id).parent;
    ensure(
        "update_negative_parent_is_stored_as_zero",
        stored == PostParent::NONE,
        || format!("{stored:?}"),
    );

    // update_null_parent_is_stored_as_zero: an `UPDATE` of `NULL` is a warning outside strict mode, and stores 0.
    repo.update(
        id,
        &PostRow::from_post(&Post {
            parent: PostParent::Int(5),
            ..changed.clone()
        }),
    )
    .expect("update")
    .written();
    let nulled = repo
        .update(
            id,
            &PostRow::from_post(&Post {
                parent: PostParent::Null,
                ..changed.clone()
            }),
        )
        .expect("update")
        .written();
    let stored = find(id).parent;
    ensure(
        "update_null_parent_is_stored_as_zero",
        nulled == Some(1) && stored == PostParent::NONE,
        || format!("{nulled:?} {stored:?}"),
    );

    // update_keeps_a_dropped_column: a callback that unsets `post_content` leaves the stored content alone.
    let before = find(id);
    let mut dropped = PostRow::from_post(&Post {
        content: b"overwritten".to_vec(),
        title: b"Retitled".to_vec(),
        ..before.clone()
    });
    dropped.remove("post_content");
    let retitled = repo.update(id, &dropped).expect("update").written();
    let after = find(id);
    ensure(
        "update_keeps_a_dropped_column",
        retitled == Some(1)
            && after.content == before.content
            && after.title == b"Retitled".to_vec(),
        || format!("{retitled:?} {after:?}"),
    );

    // update_writes_an_added_column: a callback that adds `comment_count` writes it.
    let counted = repo
        .update(id, &PostRow::new().with("comment_count", RowValue::Int(3)))
        .expect("update")
        .written();
    let count = find(id).comment_count;
    ensure(
        "update_writes_an_added_column",
        counted == Some(1) && count == 3,
        || format!("{counted:?} {count}"),
    );

    // update_of_an_unknown_column_or_an_empty_row_is_none: MySQL rejects both and changes nothing.
    let before = find(id);
    let unknown = repo
        .update(
            id,
            &PostRow::new()
                .with("post_title", RowValue::Str(b"Not written".to_vec()))
                .with("bogus", RowValue::Str(b"1".to_vec())),
        )
        .expect("update")
        .written();
    let empty = repo.update(id, &PostRow::new()).expect("update").written();
    let after = find(id);
    ensure(
        "update_of_an_unknown_column_or_an_empty_row_is_none",
        unknown.is_none() && empty.is_none() && after == before,
        || format!("{unknown:?} {empty:?} {after:?}"),
    );

    // update_null_in_a_text_column_stores_its_default: `NULL` in an `UPDATE` is the implicit default, `''`.
    let nulled = repo
        .update(id, &PostRow::new().with("post_excerpt", RowValue::Null))
        .expect("update")
        .written();
    let excerpt = find(id).excerpt;
    ensure(
        "update_null_in_a_text_column_stores_its_default",
        nulled == Some(1) && excerpt.is_empty(),
        || format!("{nulled:?} {excerpt:?}"),
    );

    // update_with_id_in_the_row_sets_it: ``SET `ID` = %d … WHERE `ID` = %d``.
    let moved_to = PostId(8_800);
    let moved = repo
        .update(
            id,
            &PostRow::new().with("ID", RowValue::Int(as_int(moved_to))),
        )
        .expect("update")
        .written();
    let at_old = repo.find(id).expect("find");
    let at_new = repo.find(moved_to).expect("find");
    ensure(
        "update_with_id_in_the_row_sets_it",
        moved == Some(1) && at_old.is_none() && at_new.is_some(),
        || format!("{moved:?} {at_old:?} {at_new:?}"),
    );

    // a_failed_update_carries_last_error
    let failed = repo
        .update(moved_to, &PostRow::new().with("bogus", RowValue::Int(1)))
        .expect("update");
    ensure(
        "a_failed_update_carries_last_error",
        failed.last_error().is_some_and(|e| !e.is_empty()),
        || format!("{failed:?}"),
    );

    column_updates(repo);
    deleting(repo);
}

/// The one-column updates: each writes its column and nothing else, and counts the rows it changed.
fn column_updates(repo: &dyn PostRepository) {
    let base = Post {
        guid: b"http://example.org/?p=old".to_vec(),
        pinged: b"http://old.example/".to_vec(),
        status: PostStatusName::new("draft"),
        ..named("wr-column", "wr-col", 0)
    };
    let id = repo
        .insert(&PostRow::from_post(&base))
        .expect("insert")
        .written()
        .expect("the insert returns an id");
    let missing = PostId(9_996);
    let read = || repo.find(id).expect("find").expect("the post exists");
    let mut expected = Post { id, ..base };

    type Check<'a> = (
        &'a str,
        Box<dyn Fn(PostId) -> Option<u64> + 'a>,
        Box<dyn Fn(&mut Post) + 'a>,
    );
    let checks: Vec<Check<'_>> = vec![
        (
            "update_name_writes_post_name",
            Box::new(|id| {
                repo.update_name(id, &slug("wr-column__trashed"))
                    .expect("update_name")
            }),
            Box::new(|p: &mut Post| p.name = slug("wr-column__trashed")),
        ),
        (
            "update_guid_writes_guid",
            Box::new(|id| {
                repo.update_guid(id, b"http://example.org/?p=new")
                    .expect("update_guid")
            }),
            Box::new(|p: &mut Post| p.guid = b"http://example.org/?p=new".to_vec()),
        ),
        (
            "update_status_writes_post_status",
            Box::new(|id| {
                repo.update_status(id, &PostStatusName::new("publish"))
                    .expect("update_status")
            }),
            Box::new(|p: &mut Post| p.status = PostStatusName::new("publish")),
        ),
        (
            "update_pinged_writes_pinged",
            Box::new(|id| {
                repo.update_pinged(id, b"http://old.example/\nhttp://new.example/")
                    .expect("update_pinged")
            }),
            Box::new(|p: &mut Post| {
                p.pinged = b"http://old.example/\nhttp://new.example/".to_vec();
            }),
        ),
        (
            "update_post_type_writes_post_type",
            Box::new(|id| {
                repo.update_post_type(as_int(id), &post_type("wr-col-2"))
                    .expect("update_post_type")
            }),
            Box::new(|p: &mut Post| p.post_type = post_type("wr-col-2")),
        ),
    ];
    // update_post_type_of_a_negative_id_changes_nothing: `WHERE `ID` = -3` matches no row.
    let negative = repo
        .update_post_type(-3, &post_type("wr-col-2"))
        .expect("update_post_type");
    ensure(
        "update_post_type_of_a_negative_id_changes_nothing",
        negative == Some(0),
        || format!("{negative:?}"),
    );

    for (check, update, apply) in &checks {
        let first = update(id);
        apply(&mut expected);
        let after = read();
        let again = update(id);
        let nobody = update(missing);
        ensure(
            check,
            first == Some(1) && after == expected && again == Some(0) && nobody == Some(0),
            || format!("{first:?} {again:?} {nobody:?} {after:?}"),
        );
    }
}

/// `wp_delete_post()`'s statements and `wp_delete_auto_drafts()`' one.
fn deleting(repo: &dyn PostRepository) {
    let insert = |post: &Post| {
        repo.insert(&PostRow::from_post(post))
            .expect("insert")
            .written()
            .expect("the insert returns an id")
    };
    let parent_of = |id: PostId| {
        repo.find(id)
            .expect("find")
            .expect("the post exists")
            .parent
    };
    let missing = PostId(9_996);

    // raw_row_reads_the_row
    let tree = insert(&named("wr-tree", "wr-tree", 77));
    let raw = repo.raw_row(tree).expect("raw_row");
    let found = repo.find(tree).expect("find");
    let none = repo.raw_row(missing).expect("raw_row");
    ensure(
        "raw_row_reads_the_row",
        raw.is_some() && raw == found && none.is_none(),
        || format!("{raw:?} {found:?} {none:?}"),
    );

    // children_of_type_reads_the_children_of_one_type
    let from = as_int(tree);
    let first = insert(&named("wr-leaf-1", "wr-tree", from));
    let second = insert(&named("wr-leaf-2", "wr-tree", from));
    let other = insert(&named("wr-leaf-3", "wr-other-tree", from));
    let attachment = insert(&named("wr-file", "attachment", from));
    let revision = insert(&named("wr-tree-revision", "revision", from));
    let mut children: Vec<PostId> = repo
        .children_of_type(tree, &post_type("wr-tree"))
        .expect("children_of_type")
        .into_iter()
        .map(|p| p.id)
        .collect();
    children.sort();
    let no_children = repo
        .children_of_type(missing, &post_type("wr-tree"))
        .expect("children_of_type");
    ensure(
        "children_of_type_reads_the_children_of_one_type",
        children == vec![first, second] && no_children.is_empty(),
        || format!("{children:?} {no_children:?}"),
    );

    // revision_ids_lists_the_revisions
    let revisions = repo.revision_ids(tree).expect("revision_ids");
    let no_revisions = repo.revision_ids(missing).expect("revision_ids");
    ensure(
        "revision_ids_lists_the_revisions",
        revisions == vec![revision] && no_revisions.is_empty(),
        || format!("{revisions:?} {no_revisions:?}"),
    );

    // reparent_children_moves_the_children_of_one_type_up
    let moved = repo
        .reparent_children(tree, PostParent::Int(77), &post_type("wr-tree"))
        .expect("reparent_children");
    let again = repo
        .reparent_children(tree, PostParent::Int(77), &post_type("wr-tree"))
        .expect("reparent_children");
    ensure(
        "reparent_children_moves_the_children_of_one_type_up",
        moved == Some(2)
            && again == Some(0)
            && parent_of(first) == PostParent::Int(77)
            && parent_of(second) == PostParent::Int(77)
            && parent_of(other) == PostParent::Int(from)
            && parent_of(attachment) == PostParent::Int(from),
        || format!("{moved:?} {again:?}"),
    );

    // reparent_attachments_moves_only_attachments_up
    let moved = repo
        .reparent_attachments(tree, PostParent::Int(77))
        .expect("reparent_attachments");
    let again = repo
        .reparent_attachments(tree, PostParent::Int(77))
        .expect("reparent_attachments");
    ensure(
        "reparent_attachments_moves_only_attachments_up",
        moved == Some(1)
            && again == Some(0)
            && parent_of(attachment) == PostParent::Int(77)
            && parent_of(other) == PostParent::Int(from)
            && parent_of(revision) == PostParent::Int(from),
        || format!("{moved:?} {again:?}"),
    );

    // remove_deletes_the_row
    let removed = repo.remove(tree).expect("remove");
    let gone = repo.find(tree).expect("find");
    let again = repo.remove(tree).expect("remove");
    let kept = repo.find(first).expect("find");
    ensure(
        "remove_deletes_the_row",
        removed == Some(1) && gone.is_none() && again == Some(0) && kept.is_some(),
        || format!("{removed:?} {gone:?} {again:?}"),
    );

    // auto_drafts_older_than_a_week_lists_old_and_zero_dated_auto_drafts: `NOW()` is the engine's clock, so the
    // dates are far from it either way.
    let dated = |status: &str, local: &str| Post {
        date: PostDate {
            local: PostDateTime::new(local),
            gmt: PostDateTime::ZERO,
        },
        ..with_status("wr-draft", "wr-draft", status)
    };
    let before = sorted(
        repo.auto_drafts_older_than_a_week()
            .expect("auto_drafts_older_than_a_week"),
    );
    let old = insert(&dated("auto-draft", "2001-01-01 00:00:00"));
    let zero = insert(&dated("auto-draft", PostDateTime::ZERO.as_str()));
    insert(&dated("auto-draft", "2999-01-01 00:00:00"));
    insert(&dated("draft", "2001-01-01 00:00:00"));
    let after = sorted(
        repo.auto_drafts_older_than_a_week()
            .expect("auto_drafts_older_than_a_week"),
    );
    let added: Vec<PostId> = after
        .iter()
        .filter(|id| !before.contains(id))
        .copied()
        .collect();
    ensure(
        "auto_drafts_older_than_a_week_lists_old_and_zero_dated_auto_drafts",
        sorted(added.clone()) == sorted(vec![old, zero])
            && before.iter().all(|id| after.contains(id)),
        || format!("{before:?} {after:?}"),
    );
}

/// Checks for repositories that behave as MySQL does on stock tables (the SQL repository); `post_repository_suite`
/// leaves them out, because the memory repository compares bytes and does no date arithmetic. Like
/// `post_repository_suite`, it needs an empty table.
///
/// - `post_name` compares as the `utf8mb4_unicode_520_ci` collation does: `get_page_by_path( 'About' )` finds `about`.
/// - `DATE_ADD` on a zero date is `NULL`, so `last_post_time` for [`Timezone::Server`] answers `None` when the newest
///   post's GMT date is zero.
pub fn mysql_suite(repo: &dyn PostRepository) {
    let insert = |post: &Post| {
        repo.insert(&PostRow::from_post(post))
            .expect("insert")
            .written()
            .expect("the insert returns an id")
    };

    // pages_by_names_ignores_case
    let about = insert(&named("about", "page", 0));
    let pages = repo
        .pages_by_names(&[slug("About")], &[post_type("page")])
        .expect("pages_by_names");
    let found: Vec<(PostId, Slug)> = pages.into_iter().map(|p| (p.id, p.name)).collect();
    ensure(
        "pages_by_names_ignores_case",
        found == vec![(about, slug("about"))],
        || format!("{found:?}"),
    );

    // server_time_of_a_zero_gmt_date_is_none
    insert(&Post {
        date: PostDate {
            local: PostDateTime::new("2024-01-01 00:00:00"),
            gmt: PostDateTime::ZERO,
        },
        ..with_status("zero-gmt", "rd-server", "publish")
    });
    let server = repo
        .last_post_time(
            LastPostField::Date,
            &[post_type("rd-server")],
            Timezone::Server,
        )
        .expect("last_post_time");
    ensure(
        "server_time_of_a_zero_gmt_date_is_none",
        server.is_none(),
        || format!("{server:?}"),
    );
}
