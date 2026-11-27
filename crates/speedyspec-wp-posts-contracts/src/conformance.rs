//! The checks every post repository passes, so the memory and SQL repositories answer alike.

use crate::{
    CommentStatus, LastPostField, MimeType, PageRow, PingStatus, Post, PostDate, PostDateTime,
    PostId, PostRepository, PostStatusName, PostTypeName, Slug, Timezone,
};

fn ensure(check: &str, ok: bool, detail: impl FnOnce() -> String) {
    assert!(ok, "post repository check `{check}` failed: {}", detail());
}

fn slug(s: &str) -> Slug {
    Slug::from(s)
}

fn post_type(s: &str) -> PostTypeName {
    PostTypeName::new(s)
}

/// A post of `post_type` named `name` under `parent`, with an auto id.
fn named(name: &str, kind: &str, parent: u64) -> Post {
    Post {
        name: slug(name),
        post_type: post_type(kind),
        parent: PostId(parent),
        ..Post::default()
    }
}

/// Runs every check against `repo`, which must start with an empty table and a fresh auto-increment
/// (`insert_with_id_is_honoured` expects 101 after an explicit 100). Panics naming the failing check.
///
/// The read statements' checks run last, on post types of their own (`rd-*`); the ones that read every page or
/// every attachment compare what the table held before and after their inserts.
pub fn post_repository_suite(repo: &dyn PostRepository) {
    let insert = |post: &Post| {
        repo.insert(post)
            .expect("insert")
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

    // insert_with_id_is_honoured: the auto-increment continues past an explicit ID, as MySQL's does.
    let explicit = insert(&Post {
        id: PostId(100),
        ..named("imported", "post", 0)
    });
    let next = insert(&named("after-import", "post", 0));
    ensure(
        "insert_with_id_is_honoured",
        explicit == PostId(100) && next == PostId(101),
        || format!("{explicit:?} {next:?}"),
    );

    // insert_taken_id_is_none
    let taken = repo
        .insert(&Post {
            id: PostId(100),
            ..named("duplicate", "post", 0)
        })
        .expect("insert");
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
        parent: b,
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

    // insert_does_not_write_comment_count
    let counted = insert(&Post {
        comment_count: 5,
        ..named("counted", "post", 0)
    });
    let count = find(counted).map(|p| p.comment_count);
    ensure(
        "insert_does_not_write_comment_count",
        count == Some(0),
        || format!("{count:?}"),
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
    let hier = |name: &str, kind: &str, except: PostId, parent: u64| {
        repo.hierarchical_slug_owner(&slug(name), &post_type(kind), except, PostId(parent))
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
        .hierarchical_slug_owner(&slug(""), &post_type("post"), PostId(0), PostId(0))
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
        .insert(&post)
        .expect("insert")
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
    mime_type: Option<&str>,
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
        repo.insert(post)
            .expect("insert")
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
    let child = insert(&named("child", "rd-many", zero_dated.0));
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
            ..named("child", "rd-many", zero_dated.0)
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
        parent: published,
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
    let private = [PostStatusName::new("private")];
    let all = counts(
        repo.count_by_status(&post_type("rd-count"), None)
            .expect("count_by_status"),
    );
    let own = counts(
        repo.count_by_status(&post_type("rd-count"), Some((7, &private)))
            .expect("count_by_status"),
    );
    let none_own = counts(
        repo.count_by_status(&post_type("rd-count"), Some((9, &private)))
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

    // count_attachments_counts_by_mime_type_and_trash: `mime_type` filters both statements.
    let filters = [
        None,
        Some("image"),
        Some("image/png"),
        Some("image/png, text"),
    ];
    let before: Vec<_> = filters
        .iter()
        .map(|f| attachment_counts(repo, *f))
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
        .map(|f| attachment_counts(repo, *f))
        .collect();
    let delta: Vec<(Vec<(String, u64)>, u64)> = before
        .iter()
        .zip(&after)
        .map(|((b, bt), (a, at))| (grown(b, a), at - bt))
        .collect();
    let mime = |m: &str, n: u64| (m.to_string(), n);
    let expected = vec![
        (vec![mime("image/jpeg", 1), mime("image/png", 2)], 2),
        (vec![mime("image/jpeg", 1), mime("image/png", 2)], 1),
        (vec![mime("image/png", 2)], 1),
        (vec![mime("image/png", 2)], 2),
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
        repo.insert(post)
            .expect("insert")
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

    // last_post_time_orders_by_the_gmt_column: the newest published post by GMT, whichever column is read.
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
            && server == at("2024-03-01 20:00:00")
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

/// Checks for repositories that compare `post_name` as MySQL's `utf8mb4_unicode_520_ci` collation does (the SQL
/// repository on stock tables); `post_repository_suite` leaves them out, because the memory repository compares
/// bytes. `get_page_by_path( 'About' )` finds `about` this way. Like `post_repository_suite`, it needs an empty table.
pub fn collation_suite(repo: &dyn PostRepository) {
    // pages_by_names_ignores_case
    let about = repo
        .insert(&named("about", "page", 0))
        .expect("insert")
        .expect("the insert returns an id");
    let pages = repo
        .pages_by_names(&[slug("About")], &[post_type("page")])
        .expect("pages_by_names");
    let found: Vec<(PostId, Slug)> = pages.into_iter().map(|p| (p.id, p.name)).collect();
    ensure(
        "pages_by_names_ignores_case",
        found == vec![(about, slug("about"))],
        || format!("{found:?}"),
    );
}
