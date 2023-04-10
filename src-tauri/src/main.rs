// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use copypasta::{ClipboardContext, ClipboardProvider};
use lazy_static::lazy_static;
use sqlite::Connection;
use std::{sync::Mutex, time::SystemTime};
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

struct LocalDBHistory(Mutex<Connection>);

lazy_static! {
    static ref CLIPBOARD_HISTORY: ClipboardHistory = ClipboardHistory(Mutex::new(vec![]));
    static ref LOCAL_DB_CONNECTION: LocalDBHistory = LocalDBHistory(Mutex::new(
        sqlite::open("./copy-camel.db").expect("Unable to connect to local DB")
    ));
}

#[tauri::command]
fn retrieve_history() -> Vec<String> {
    let history: Vec<String> = CLIPBOARD_HISTORY.retrieve();
    history
}

#[tauri::command]
fn set_contents(new_contents: String) -> Result<(), ()> {
    let mut clipboard_ctx: ClipboardContext = ClipboardContext::new().unwrap();
    println!("lololol");
    clipboard_ctx.set_contents(new_contents);
    Ok(())
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
                        let clipboard_time = SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        let query = format!(
                            "INSERT INTO history(entry, timestamp) VALUES ('{}', {})",
                            last_contents, clipboard_time
                        );
                        LOCAL_DB_CONNECTION
                            .0
                            .lock()
                            .unwrap()
                            .execute(query)
                            .expect("Unable to append entry to local DB");
                    }
                }
                _ => {}
            }
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
    });
}

fn main() {
    let db_clear_shortcut = CustomMenuItem::new(String::from("cleardb"), "Clear History");
    let quit_shortcut = CustomMenuItem::new(String::from("quit"), "Quit").accelerator("Cmd+Q");
    let system_tray_menu = SystemTrayMenu::new()
        .add_item(db_clear_shortcut)
        .add_item(quit_shortcut);

    init_polling();

    let query = "
        CREATE TABLE IF NOT EXISTS history (id INTEGER PRIMARY KEY AUTOINCREMENT, entry TEXT, timestamp INTEGER);
    ";
    LOCAL_DB_CONNECTION
        .0
        .lock()
        .unwrap()
        .execute(query)
        .unwrap();

    let mut app = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![retrieve_history, set_contents])
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
                    "cleardb" => {
                        let query = "
                            DELETE FROM history;
                        ";

                        LOCAL_DB_CONNECTION
                            .0
                            .lock()
                            .unwrap()
                            .execute(query)
                            .expect("Unable to clear history DB");
                    }
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
        .build(tauri::generate_context!())
        .expect("Error while building application");

    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);

    app.run(|_app_handle, _event| {});
}
