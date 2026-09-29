use super::*;

#[test]
fn a_site_is_the_host_without_www() {
    assert_eq!(
        site_of("https://www.Wikipedia.org/wiki/X").as_deref(),
        Some("wikipedia.org")
    );
    assert_eq!(
        site_of("http://user:pw@pt.wikipedia.org:8080/?q#f").as_deref(),
        Some("pt.wikipedia.org")
    );
    assert_eq!(
        site_of("http://127.0.0.1:5173/").as_deref(),
        Some("127.0.0.1")
    );
    assert_eq!(site_of("http://[::1]:80/").as_deref(), Some("::1"));
    assert_eq!(site_of("http://localhost").as_deref(), Some("localhost"));
    assert_eq!(site_of("about:blank"), None);
    assert_eq!(site_of("file:///C:/x.html"), None);
    assert_eq!(site_of("https:///nothing"), None);
}

#[test]
fn allowing_a_site_allows_its_subdomains_only() {
    assert!(covers("wikipedia.org", "wikipedia.org"));
    assert!(covers("wikipedia.org", "pt.wikipedia.org"));
    assert!(!covers("pt.wikipedia.org", "wikipedia.org"));
    assert!(!covers("wikipedia.org", "notwikipedia.org"));
}

#[test]
fn web_addresses_and_bare_hosts_open_as_web() {
    let dir = tempfile::tempdir().expect("tempdir");
    let base = dir.path();
    assert_eq!(
        resolve("https://example.com/a?b", base, &[]),
        Ok(Place::Web {
            url: "https://example.com/a?b".to_owned(),
            site: Some("example.com".to_owned()),
        })
    );
    assert_eq!(
        resolve("example.com/docs", base, &[]),
        Ok(Place::Web {
            url: "https://example.com/docs".to_owned(),
            site: Some("example.com".to_owned()),
        })
    );
    assert_eq!(
        resolve("about:blank", base, &[]),
        Ok(Place::Web {
            url: "about:blank".to_owned(),
            site: None,
        })
    );
}

#[test]
fn other_schemes_are_refused() {
    let dir = tempfile::tempdir().expect("tempdir");
    for input in [
        "javascript:alert(1)",
        "data:text/html,hi",
        "edge://settings",
        "chrome://version",
        "mailto:a@b.c",
        "about:config",
        "not a place",
    ] {
        assert!(
            resolve(input, dir.path(), &[]).is_err(),
            "{input} was accepted"
        );
    }
}

#[test]
fn files_open_only_inside_the_bot_folders() {
    let dir = tempfile::tempdir().expect("tempdir");
    let bot = dir.path().join("bot");
    let work = dir.path().join("work folder");
    let elsewhere = dir.path().join("elsewhere");
    for folder in [&bot, &work, &elsewhere] {
        std::fs::create_dir_all(folder).expect("folder");
    }
    std::fs::write(bot.join("page.html"), "<p>hi</p>").expect("write");
    std::fs::write(work.join("site é.html"), "<p>hi</p>").expect("write");
    std::fs::write(elsewhere.join("x.html"), "<p>no</p>").expect("write");
    let folders = [bot.clone(), work.clone()];

    let Ok(Place::File { url }) = resolve("page.html", &bot, &folders) else {
        panic!("a relative file in the bot's folder opens");
    };
    assert!(url.starts_with("file:///"), "{url}");
    assert!(url.ends_with("/bot/page.html"), "{url}");
    assert!(file_allowed(&url, &folders));

    let absolute = work.join("site é.html");
    let Ok(Place::File { url }) = resolve(&absolute.to_string_lossy(), &bot, &folders) else {
        panic!("a file in the work folder opens");
    };
    assert!(url.contains("work%20folder/site%20%C3%A9.html"), "{url}");
    assert_eq!(
        resolve(&url, &bot, &folders),
        Ok(Place::File { url: url.clone() })
    );

    let outside = elsewhere.join("x.html");
    assert!(resolve(&outside.to_string_lossy(), &bot, &folders).is_err());
    assert!(resolve(r"..\elsewhere\x.html", &bot, &folders).is_err());
    assert!(!file_allowed(
        &format!("file:///{}", outside.to_string_lossy().replace('\\', "/")),
        &folders
    ));
    assert!(resolve("missing.html", &bot, &folders).is_err());
}
