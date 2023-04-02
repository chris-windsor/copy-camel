// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use copypasta::{ClipboardContext, ClipboardProvider};
use lazy_static::lazy_static;
use std::sync::Mutex;
use tauri::{CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu};
use tauri_plugin_positioner::WindowExt;

struct ClipboardHistory(Mutex<Vec<String>>);

impl ClipboardHistory {
    fn add_entry(&self, new_value: String) {
        let _ = &self.0.lock().unwrap().push(new_value);
    }

    fn retrieve(&self) -> Vec<String> {
        (&self.0.lock().unwrap()).to_vec()
    }
}

lazy_static! {
    static ref CLIPBOARD_HISTORY: ClipboardHistory = ClipboardHistory(Mutex::new(vec![]));
}

#[tauri::command]
fn greet(content: &str) -> String {
    let mut ctx = ClipboardContext::new().unwrap();

    ctx.set_contents(content.to_owned()).unwrap();

    println!("{:#?}", CLIPBOARD_HISTORY.retrieve());

    format!("Hey! I copied '{}' to your clipboard", content)
}

fn init_polling() {
    std::thread::spawn(move || {
        let mut clipboard_ctx: ClipboardContext = ClipboardContext::new().unwrap();

        let mut last_contents = String::new();

        loop {
            let clipboard_contents = clipboard_ctx.get_contents();

            match clipboard_contents {
                Ok(contents) => {
                    if contents != last_contents {
                        last_contents = contents.clone();
                        CLIPBOARD_HISTORY.add_entry(last_contents.clone());
                    }
                }
                _ => {}
            }
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
    });
}

fn main() {
    let quit_shortcut = CustomMenuItem::new(String::from("quit"), "Quit").accelerator("Cmd+Q");
    let system_tray_menu = SystemTrayMenu::new().add_item(quit_shortcut);

    init_polling();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .plugin(tauri_plugin_positioner::init())
        .system_tray(SystemTray::new().with_menu(system_tray_menu))
        .on_system_tray_event(|app, event| {
            tauri_plugin_positioner::on_tray_event(app, &event);
            match event {
                SystemTrayEvent::LeftClick { .. } => {
                    let window = app.get_window("main").unwrap();
                    let _ = window.move_window(tauri_plugin_positioner::Position::TrayCenter);

                    if window.is_visible().unwrap() {
                        window.hide().unwrap();
                    } else {
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                }
                SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                    "quit" => {
                        std::process::exit(0);
                    }
                    _ => {}
                },
                _ => {}
            }
        })
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::Focused(is_focused) => {
                if !is_focused {
                    event.window().hide().unwrap();
                }
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
