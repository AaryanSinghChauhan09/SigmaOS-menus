//! Window management with advanced features
//! Follows: Component Isolation, Single Responsibility, Interface Segregation

use std::collections::HashMap;

/// Window position and size
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Rect { x, y, width, height }
    }

    pub fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x && px < self.x + self.width as i32 &&
        py >= self.y && py < self.y + self.height as i32
    }
}

/// Window state
#[derive(Debug, Clone, PartialEq)]
pub enum WindowState {
    Normal,
    Minimized,
    Maximized,
    Fullscreen,
    Hidden,
}

/// Window type
#[derive(Debug, Clone, PartialEq)]
pub enum WindowType {
    Normal,
    Dialog,
    Dock,
    Toolbar,
    Menu,
    Utility,
    Splash,
}

/// Desktop window
pub struct Window {
    pub id: u64,
    pub title: String,
    pub rect: Rect,
    pub state: WindowState,
    pub window_type: WindowType,
    pub focused: bool,
}

impl Window {
    pub fn new(id: u64, title: String, rect: Rect) -> Self {
        Window {
            id,
            title,
            rect,
            state: WindowState::Normal,
            window_type: WindowType::Normal,
            focused: false,
        }
    }

    pub fn set_state(&mut self, state: WindowState) {
        self.state = state;
    }

    pub fn focus(&mut self) {
        self.focused = true;
    }

    pub fn unfocus(&mut self) {
        self.focused = false;
    }
}

/// Window manager
pub struct WindowManager {
    windows: HashMap<u64, Window>,
    focused_window: Option<u64>,
    next_id: u64,
}

impl WindowManager {
    pub fn new() -> Self {
        WindowManager {
            windows: HashMap::new(),
            focused_window: None,
            next_id: 1,
        }
    }

    pub fn create_window(&mut self, title: String, rect: Rect) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let window = Window::new(id, title, rect);
        self.windows.insert(id, window);
        id
    }

    pub fn focus_window(&mut self, id: u64) -> Result<(), &'static str> {
        if !self.windows.contains_key(&id) {
            return Err("Window not found");
        }
        
        // Unfocus current
        if let Some(current_id) = self.focused_window {
            if let Some(window) = self.windows.get_mut(&current_id) {
                window.unfocus();
            }
        }
        
        // Focus new
        if let Some(window) = self.windows.get_mut(&id) {
            window.focus();
        }
        
        self.focused_window = Some(id);
        Ok(())
    }

    pub fn get_window(&self, id: u64) -> Option<&Window> {
        self.windows.get(&id)
    }

    pub fn close_window(&mut self, id: u64) -> Result<(), &'static str> {
        if !self.windows.contains_key(&id) {
            return Err("Window not found");
        }
        
        if self.focused_window == Some(id) {
            self.focused_window = None;
        }
        
        self.windows.remove(&id);
        Ok(())
    }
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_creation() {
        let mut wm = WindowManager::new();
        let rect = Rect::new(0, 0, 800, 600);
        let id = wm.create_window("Test".to_string(), rect);
        assert!(wm.get_window(id).is_some());
    }

    #[test]
    fn test_window_focus() {
        let mut wm = WindowManager::new();
        let rect = Rect::new(0, 0, 800, 600);
        let id = wm.create_window("Test".to_string(), rect);
        wm.focus_window(id).unwrap();
        assert!(wm.get_window(id).unwrap().focused);
    }
}
