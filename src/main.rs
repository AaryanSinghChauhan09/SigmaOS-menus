mod config;
mod error;
mod service;
mod window;
mod file;
mod theme;

use config::Config;
use service::Service;
use window::{WindowManager, Rect};
use file::FileManager;
use theme::{ThemeManager, Theme, ColorPalette, Color};
use anyhow::Result;

fn main() -> Result<()> {
    env_logger::init();
    
    let config = Config::default();
    let mut service = Service::new(config);
    
    service.start()?;
    println!("Service started successfully");
    
    // Demonstrate window management
    let mut wm = WindowManager::new();
    let rect = Rect::new(100, 100, 800, 600);
    let window_id = wm.create_window("Test Window".to_string(), rect);
    wm.focus_window(window_id)?;
    println!("Window created and focused: {}", window_id);
    
    // Demonstrate file management
    let fm = FileManager::default();
    let files = fm.list_directory()?;
    println!("Directory listing: {} files", files.len());
    
    // Demonstrate theme management
    let mut tm = ThemeManager::new();
    let theme = Theme::new("SigmaOS Default".to_string());
    tm.add_theme(theme);
    tm.set_current_theme("SigmaOS Default")?;
    println!("Theme set successfully");
    
    service.stop()?;
    println!("Service stopped successfully");
    
    Ok(())
}
