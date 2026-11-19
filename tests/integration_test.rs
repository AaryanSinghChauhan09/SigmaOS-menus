use sigmaos_desktop_component::{DesktopEntry, MenuDatabase, Category};

#[test]
fn integration_full_menu_workflow() {
    // Arrange: build a database with several apps
    let mut db = MenuDatabase::new();
    let firefox = DesktopEntry::new("firefox", "Firefox", "firefox %u")
        .unwrap()
        .with_categories("Network;WebBrowser;")
        .unwrap()
        .with_keywords("web;browser;internet");
    let gimp = DesktopEntry::new("gimp", "GIMP", "gimp %U")
        .unwrap()
        .with_categories("Graphics;2DGraphics;")
        .unwrap();
    let kitty = DesktopEntry::new("kitty", "Kitty", "kitty")
        .unwrap()
        .with_categories("System;TerminalEmulator;")
        .unwrap();
    db.insert(firefox).unwrap();
    db.insert(gimp).unwrap();
    db.insert(kitty).unwrap();

    // Act & Assert: search
    let results = db.search("kit").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Kitty");

    // Act & Assert: by category
    let cats = db.by_category();
    assert!(cats.contains_key(&Category::Network));
    assert!(cats.contains_key(&Category::Graphics));
    assert!(cats.contains_key(&Category::System));

    // Act & Assert: all visible
    let all = db.all_visible();
    assert_eq!(all.len(), 3);
    // Should be sorted by name
    assert_eq!(all[0].name, "Firefox");
    assert_eq!(all[1].name, "GIMP");
    assert_eq!(all[2].name, "Kitty");
}

#[test]
fn integration_remove_and_search() {
    let mut db = MenuDatabase::new();
    db.insert(DesktopEntry::new("app1", "App One", "app1").unwrap()).unwrap();
    db.insert(DesktopEntry::new("app2", "App Two", "app2").unwrap()).unwrap();
    assert_eq!(db.len(), 2);

    db.remove("app1");
    assert_eq!(db.len(), 1);
    assert!(db.search("One").unwrap().is_empty());
    assert_eq!(db.search("Two").unwrap().len(), 1);
}
