// SPDX-License-Identifier: MIT OR Apache-2.0
//! SigmaOS-menus — Application menu & menu database.
//!
//! XDG-compliant application menu system inspired by Omarchy/Hyprland launchers
//! (Rofi, Wofi, Fuzzel). Provides desktop entry parsing, hierarchical menu tree
//! building, keyword search/filtering, and category-based organisation.
//!
//! Principles: fail-fast validation, deterministic ordering, zero `unsafe`.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fmt;

// ── Errors ──────────────────────────────────────────────────────────────

/// Errors emitted by the menu subsystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuError {
    /// Desktop entry is missing the required `Name` field.
    MissingName,
    /// Desktop entry is missing the required `Exec` field.
    MissingExec,
    /// Category string is empty.
    EmptyCategory,
    /// Duplicate entry id.
    DuplicateEntry(String),
    /// Search query is empty.
    EmptyQuery,
}

impl fmt::Display for MenuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingName => write!(f, "desktop entry missing required Name field"),
            Self::MissingExec => write!(f, "desktop entry missing required Exec field"),
            Self::EmptyCategory => write!(f, "category string must not be empty"),
            Self::DuplicateEntry(id) => write!(f, "duplicate desktop entry id: {id}"),
            Self::EmptyQuery => write!(f, "search query must not be empty"),
        }
    }
}

impl std::error::Error for MenuError {}

// ── XDG Categories ──────────────────────────────────────────────────────

/// Standard XDG main categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Category {
    AudioVideo,
    Development,
    Education,
    Game,
    Graphics,
    Network,
    Office,
    Science,
    Settings,
    System,
    Utility,
    Other,
}

impl Category {
    /// Parse a category string (case-insensitive).
    pub fn parse(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "audiovideo" | "audio" | "video" | "multimedia" => Self::AudioVideo,
            "development" | "ide" | "programming" => Self::Development,
            "education" => Self::Education,
            "game" | "games" => Self::Game,
            "graphics" | "2dgraphics" | "3dgraphics" => Self::Graphics,
            "network" | "webbrowser" | "email" => Self::Network,
            "office" | "wordprocessor" | "spreadsheet" => Self::Office,
            "science" | "math" | "astronomy" => Self::Science,
            "settings" | "desktopsettings" => Self::Settings,
            "system" | "monitor" | "terminalemulator" => Self::System,
            "utility" | "texteditor" | "accessibility" => Self::Utility,
            _ => Self::Other,
        }
    }

    /// Human-readable label for the category.
    pub fn label(&self) -> &'static str {
        match self {
            Self::AudioVideo => "Audio & Video",
            Self::Development => "Development",
            Self::Education => "Education",
            Self::Game => "Games",
            Self::Graphics => "Graphics",
            Self::Network => "Network",
            Self::Office => "Office",
            Self::Science => "Science",
            Self::Settings => "Settings",
            Self::System => "System",
            Self::Utility => "Utilities",
            Self::Other => "Other",
        }
    }
}

// ── Desktop Entry ───────────────────────────────────────────────────────

/// A parsed XDG `.desktop` entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopEntry {
    /// Unique identifier (usually the .desktop filename without extension).
    pub id: String,
    /// Human-readable application name.
    pub name: String,
    /// Optional generic name (e.g. "Web Browser").
    pub generic_name: Option<String>,
    /// Exec command line.
    pub exec: String,
    /// Optional icon name or path.
    pub icon: Option<String>,
    /// Optional comment / tooltip.
    pub comment: Option<String>,
    /// Categories this entry belongs to.
    pub categories: Vec<Category>,
    /// Search keywords.
    pub keywords: Vec<String>,
    /// Whether the entry should be shown in menus.
    pub no_display: bool,
    /// Whether this is a terminal application.
    pub terminal: bool,
}

impl DesktopEntry {
    /// Create a new desktop entry with validation.
    pub fn new(id: impl Into<String>, name: impl Into<String>, exec: impl Into<String>) -> Result<Self, MenuError> {
        let name = name.into();
        let exec = exec.into();
        if name.trim().is_empty() {
            return Err(MenuError::MissingName);
        }
        if exec.trim().is_empty() {
            return Err(MenuError::MissingExec);
        }
        Ok(Self {
            id: id.into(),
            name,
            generic_name: None,
            exec,
            icon: None,
            comment: None,
            categories: vec![Category::Other],
            keywords: Vec::new(),
            no_display: false,
            terminal: false,
        })
    }

    /// Set the categories from a semicolon-separated string (XDG format).
    pub fn with_categories(mut self, cats: &str) -> Result<Self, MenuError> {
        let parsed: Vec<Category> = cats
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(Category::parse)
            .collect();
        if parsed.is_empty() {
            return Err(MenuError::EmptyCategory);
        }
        self.categories = parsed;
        Ok(self)
    }

    /// Set keywords from a semicolon-separated string.
    pub fn with_keywords(mut self, kw: &str) -> Self {
        self.keywords = kw
            .split(';')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();
        self
    }

    /// Check if this entry matches a search query (case-insensitive).
    pub fn matches(&self, query: &str) -> bool {
        let q = query.to_lowercase();
        self.name.to_lowercase().contains(&q)
            || self.generic_name.as_deref().map_or(false, |g| g.to_lowercase().contains(&q))
            || self.comment.as_deref().map_or(false, |c| c.to_lowercase().contains(&q))
            || self.keywords.iter().any(|k| k.contains(&q))
            || self.id.to_lowercase().contains(&q)
    }

    /// Primary category for tree placement.
    pub fn primary_category(&self) -> Category {
        self.categories.first().copied().unwrap_or(Category::Other)
    }
}

// ── Menu Database ───────────────────────────────────────────────────────

/// In-memory menu database: stores entries, provides search and category views.
#[derive(Debug, Default)]
pub struct MenuDatabase {
    entries: BTreeMap<String, DesktopEntry>,
}

impl MenuDatabase {
    /// Create an empty menu database.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a desktop entry. Returns error on duplicate id.
    pub fn insert(&mut self, entry: DesktopEntry) -> Result<(), MenuError> {
        if self.entries.contains_key(&entry.id) {
            return Err(MenuError::DuplicateEntry(entry.id.clone()));
        }
        self.entries.insert(entry.id.clone(), entry);
        Ok(())
    }

    /// Total number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the database is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get an entry by id.
    pub fn get(&self, id: &str) -> Option<&DesktopEntry> {
        self.entries.get(id)
    }

    /// Search entries by query string. Returns matches sorted by name.
    pub fn search(&self, query: &str) -> Result<Vec<&DesktopEntry>, MenuError> {
        if query.trim().is_empty() {
            return Err(MenuError::EmptyQuery);
        }
        let mut results: Vec<&DesktopEntry> = self
            .entries
            .values()
            .filter(|e| !e.no_display && e.matches(query))
            .collect();
        results.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(results)
    }

    /// Group visible entries by primary category.
    pub fn by_category(&self) -> BTreeMap<Category, Vec<&DesktopEntry>> {
        let mut groups: BTreeMap<Category, Vec<&DesktopEntry>> = BTreeMap::new();
        for entry in self.entries.values() {
            if entry.no_display {
                continue;
            }
            groups
                .entry(entry.primary_category())
                .or_default()
                .push(entry);
        }
        for entries in groups.values_mut() {
            entries.sort_by(|a, b| a.name.cmp(&b.name));
        }
        groups
    }

    /// List all visible entries sorted by name.
    pub fn all_visible(&self) -> Vec<&DesktopEntry> {
        let mut entries: Vec<&DesktopEntry> = self
            .entries
            .values()
            .filter(|e| !e.no_display)
            .collect();
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        entries
    }

    /// Remove an entry by id. Returns the entry if it existed.
    pub fn remove(&mut self, id: &str) -> Option<DesktopEntry> {
        self.entries.remove(id)
    }
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_desktop_entry_creation() {
        let entry = DesktopEntry::new("firefox", "Firefox", "firefox %u").unwrap();
        assert_eq!(entry.name, "Firefox");
        assert_eq!(entry.exec, "firefox %u");
        assert_eq!(entry.primary_category(), Category::Other);
    }

    #[test]
    fn test_desktop_entry_missing_name() {
        let err = DesktopEntry::new("x", "", "cmd").unwrap_err();
        assert_eq!(err, MenuError::MissingName);
    }

    #[test]
    fn test_desktop_entry_missing_exec() {
        let err = DesktopEntry::new("x", "App", "").unwrap_err();
        assert_eq!(err, MenuError::MissingExec);
    }

    #[test]
    fn test_category_parsing() {
        let entry = DesktopEntry::new("ff", "Firefox", "firefox")
            .unwrap()
            .with_categories("Network;WebBrowser;")
            .unwrap();
        assert_eq!(entry.categories, vec![Category::Network, Category::Network]);
        assert_eq!(entry.primary_category(), Category::Network);
    }

    #[test]
    fn test_keyword_search() {
        let entry = DesktopEntry::new("ff", "Firefox", "firefox")
            .unwrap()
            .with_keywords("web;browser;internet");
        assert!(entry.matches("web"));
        assert!(entry.matches("fire"));
        assert!(!entry.matches("spreadsheet"));
    }

    #[test]
    fn test_database_insert_and_search() {
        let mut db = MenuDatabase::new();
        db.insert(DesktopEntry::new("ff", "Firefox", "firefox").unwrap()).unwrap();
        db.insert(DesktopEntry::new("tb", "Thunderbird", "thunderbird").unwrap()).unwrap();
        assert_eq!(db.len(), 2);

        let results = db.search("fire").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "ff");
    }

    #[test]
    fn test_database_duplicate_reject() {
        let mut db = MenuDatabase::new();
        db.insert(DesktopEntry::new("ff", "Firefox", "firefox").unwrap()).unwrap();
        let err = db.insert(DesktopEntry::new("ff", "Firefox2", "firefox2").unwrap()).unwrap_err();
        assert_eq!(err, MenuError::DuplicateEntry("ff".into()));
    }

    #[test]
    fn test_database_by_category() {
        let mut db = MenuDatabase::new();
        db.insert(
            DesktopEntry::new("ff", "Firefox", "firefox")
                .unwrap()
                .with_categories("Network")
                .unwrap(),
        ).unwrap();
        db.insert(
            DesktopEntry::new("vim", "Vim", "vim")
                .unwrap()
                .with_categories("Utility;TextEditor")
                .unwrap(),
        ).unwrap();
        let cats = db.by_category();
        assert!(cats.contains_key(&Category::Network));
        assert!(cats.contains_key(&Category::Utility));
        assert_eq!(cats[&Category::Network].len(), 1);
    }

    #[test]
    fn test_no_display_filtered() {
        let mut db = MenuDatabase::new();
        let mut entry = DesktopEntry::new("hidden", "Hidden", "cmd").unwrap();
        entry.no_display = true;
        db.insert(entry).unwrap();
        assert_eq!(db.all_visible().len(), 0);
        assert_eq!(db.search("hidden").unwrap().len(), 0);
    }

    #[test]
    fn test_empty_search_rejected() {
        let db = MenuDatabase::new();
        assert_eq!(db.search("").unwrap_err(), MenuError::EmptyQuery);
    }
}
