//! Theme management with color schemes and styling
//! Follows: Composition Over Inheritance, Open-Closed Principle

use std::collections::HashMap;

/// Color representation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color::new(r, g, b, 255)
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

/// Theme color palette
#[derive(Debug, Clone)]
pub struct ColorPalette {
    pub primary: Color,
    pub secondary: Color,
    pub background: Color,
    pub foreground: Color,
    pub accent: Color,
}

impl Default for ColorPalette {
    fn default() -> Self {
        ColorPalette {
            primary: Color::rgb(0, 120, 215),
            secondary: Color::rgb(0, 80, 150),
            background: Color::rgb(255, 255, 255),
            foreground: Color::rgb(0, 0, 0),
            accent: Color::rgb(255, 100, 0),
        }
    }
}

/// Desktop theme
pub struct Theme {
    pub name: String,
    pub palette: ColorPalette,
    pub font_family: String,
    pub font_size: u32,
    pub border_radius: u32,
    pub custom_properties: HashMap<String, String>,
}

impl Theme {
    pub fn new(name: String) -> Self {
        Theme {
            name,
            palette: ColorPalette::default(),
            font_family: "System".to_string(),
            font_size: 12,
            border_radius: 8,
            custom_properties: HashMap::new(),
        }
    }

    pub fn set_palette(&mut self, palette: ColorPalette) {
        self.palette = palette;
    }

    pub fn set_property(&mut self, key: String, value: String) {
        self.custom_properties.insert(key, value);
    }

    pub fn get_property(&self, key: &str) -> Option<&String> {
        self.custom_properties.get(key)
    }
}

/// Theme manager
pub struct ThemeManager {
    themes: HashMap<String, Theme>,
    current_theme: Option<String>,
}

impl ThemeManager {
    pub fn new() -> Self {
        ThemeManager {
            themes: HashMap::new(),
            current_theme: None,
        }
    }

    pub fn add_theme(&mut self, theme: Theme) {
        let name = theme.name.clone();
        self.themes.insert(name, theme);
    }

    pub fn set_current_theme(&mut self, name: &str) -> Result<(), &'static str> {
        if !self.themes.contains_key(name) {
            return Err("Theme not found");
        }
        self.current_theme = Some(name.to_string());
        Ok(())
    }

    pub fn get_current_theme(&self) -> Option<&Theme> {
        self.current_theme.as_ref()
            .and_then(|name| self.themes.get(name))
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_management() {
        let mut tm = ThemeManager::new();
        let theme = Theme::new("Test".to_string());
        tm.add_theme(theme);
        tm.set_current_theme("Test").unwrap();
        assert!(tm.get_current_theme().is_some());
    }
}
