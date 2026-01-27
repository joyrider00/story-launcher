use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, Runtime, Emitter,
    image::Image,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolStatus {
    pub installed: bool,
    pub local_version: Option<String>,
    pub local_commit: Option<String>,
    pub remote_commit: Option<String>,
    pub has_update: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateResult {
    pub success: bool,
    pub message: String,
}

// Global state for tracking update availability
pub struct AppState {
    pub has_updates: Mutex<bool>,
}

fn get_tool_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_default();
    home.join("projects").join("spellbook-resolve-sync")
}

#[tauri::command]
fn check_tool_status() -> ToolStatus {
    let tool_path = get_tool_path();

    if !tool_path.exists() {
        return ToolStatus {
            installed: false,
            local_version: None,
            local_commit: None,
            remote_commit: None,
            has_update: false,
            error: None,
        };
    }

    let version_path = tool_path.join("VERSION");
    let local_version = std::fs::read_to_string(&version_path)
        .ok()
        .map(|v| v.trim().to_string());

    let local_commit = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&tool_path)
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        });

    let _ = Command::new("git")
        .args(["fetch", "origin"])
        .current_dir(&tool_path)
        .output();

    let remote_commit = Command::new("git")
        .args(["rev-parse", "origin/main"])
        .current_dir(&tool_path)
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        })
        .or_else(|| {
            Command::new("git")
                .args(["rev-parse", "origin/master"])
                .current_dir(&tool_path)
                .output()
                .ok()
                .and_then(|output| {
                    if output.status.success() {
                        String::from_utf8(output.stdout)
                            .ok()
                            .map(|s| s.trim().to_string())
                    } else {
                        None
                    }
                })
        });

    let has_update = match (&local_commit, &remote_commit) {
        (Some(local), Some(remote)) => local != remote,
        _ => false,
    };

    ToolStatus {
        installed: true,
        local_version,
        local_commit,
        remote_commit,
        has_update,
        error: None,
    }
}

#[tauri::command]
fn update_tool() -> UpdateResult {
    let tool_path = get_tool_path();

    if !tool_path.exists() {
        return UpdateResult {
            success: false,
            message: "Tool not installed".to_string(),
        };
    }

    let pull_result = Command::new("git")
        .args(["pull", "origin"])
        .current_dir(&tool_path)
        .output();

    match pull_result {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return UpdateResult {
                    success: false,
                    message: format!("Git pull failed: {}", stderr),
                };
            }
        }
        Err(e) => {
            return UpdateResult {
                success: false,
                message: format!("Failed to run git: {}", e),
            };
        }
    }

    let venv_path = tool_path.join(".venv");
    if venv_path.exists() {
        let pip_path = venv_path.join("bin").join("pip");
        let install_result = Command::new(&pip_path)
            .args(["install", "-e", "."])
            .current_dir(&tool_path)
            .output();

        match install_result {
            Ok(output) => {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return UpdateResult {
                        success: false,
                        message: format!("Pip install failed: {}", stderr),
                    };
                }
            }
            Err(e) => {
                return UpdateResult {
                    success: false,
                    message: format!("Failed to run pip: {}", e),
                };
            }
        }
    }

    UpdateResult {
        success: true,
        message: "Update complete".to_string(),
    }
}

#[tauri::command]
fn launch_tool() -> UpdateResult {
    let tool_path = get_tool_path();
    let app_path = tool_path.join("dist").join("Spellbook Resolve Sync.app");

    if app_path.exists() {
        let result = Command::new("open")
            .arg(&app_path)
            .output();

        match result {
            Ok(output) => {
                if output.status.success() {
                    return UpdateResult {
                        success: true,
                        message: "Launched app".to_string(),
                    };
                }
            }
            Err(_) => {}
        }
    }

    let venv_python = tool_path.join(".venv").join("bin").join("python");
    let main_py = tool_path.join("main.py");

    if venv_python.exists() && main_py.exists() {
        let result = Command::new(&venv_python)
            .arg(&main_py)
            .current_dir(&tool_path)
            .spawn();

        match result {
            Ok(_) => UpdateResult {
                success: true,
                message: "Launched via Python".to_string(),
            },
            Err(e) => UpdateResult {
                success: false,
                message: format!("Failed to launch: {}", e),
            },
        }
    } else {
        UpdateResult {
            success: false,
            message: "App not found".to_string(),
        }
    }
}

#[tauri::command]
fn build_tool() -> UpdateResult {
    let tool_path = get_tool_path();
    let build_script = tool_path.join("build.sh");

    if !build_script.exists() {
        return UpdateResult {
            success: false,
            message: "Build script not found".to_string(),
        };
    }

    let result = Command::new("bash")
        .arg(&build_script)
        .current_dir(&tool_path)
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                UpdateResult {
                    success: true,
                    message: "Build complete".to_string(),
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                UpdateResult {
                    success: false,
                    message: format!("Build failed: {}", stderr),
                }
            }
        }
        Err(e) => UpdateResult {
            success: false,
            message: format!("Failed to run build: {}", e),
        },
    }
}

#[tauri::command]
fn set_tray_update_icon<R: Runtime>(app: tauri::AppHandle<R>, has_update: bool) {
    if let Some(tray) = app.tray_by_id("main-tray") {
        let icon_path = if has_update {
            include_bytes!("../icons/tray-icon-update.png").to_vec()
        } else {
            include_bytes!("../icons/tray-icon.png").to_vec()
        };

        if let Ok(icon) = Image::from_bytes(&icon_path) {
            let _ = tray.set_icon(Some(icon));
        }
    }
}

fn create_tray_menu<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<Menu<R>> {
    let resolve_sync = MenuItem::with_id(app, "resolve-sync", "Resolve Sync Script", true, None::<&str>)?;
    let spellbook = MenuItem::with_id(app, "spellbook", "Spellbook", true, None::<&str>)?;
    let portal = MenuItem::with_id(app, "portal", "Story Portal", true, None::<&str>)?;
    let separator1 = PredefinedMenuItem::separator(app)?;
    let check_updates = MenuItem::with_id(app, "check-updates", "Check for Updates", true, None::<&str>)?;
    let open_launcher = MenuItem::with_id(app, "open-launcher", "Open Story Launcher", true, None::<&str>)?;
    let separator2 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Story Launcher", true, None::<&str>)?;

    Menu::with_items(app, &[
        &resolve_sync,
        &spellbook,
        &portal,
        &separator1,
        &check_updates,
        &open_launcher,
        &separator2,
        &quit,
    ])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(AppState {
            has_updates: Mutex::new(false),
        })
        .setup(|app| {
            let handle = app.handle().clone();

            // Create tray icon
            let tray_icon = Image::from_bytes(include_bytes!("../icons/tray-icon.png"))?;
            let menu = create_tray_menu(&handle)?;

            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(tray_icon)
                .menu(&menu)
                .tooltip("Story Launcher")
                .on_menu_event(move |app, event| {
                    match event.id.as_ref() {
                        "resolve-sync" => {
                            let _ = launch_tool();
                        }
                        "spellbook" => {
                            let _ = Command::new("open")
                                .arg("https://spellbook.story.inc")
                                .spawn();
                        }
                        "portal" => {
                            let _ = Command::new("open")
                                .arg("https://portal.story.inc")
                                .spawn();
                        }
                        "check-updates" => {
                            // Show the main window and trigger update check
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                                let _ = window.emit("check-updates", ());
                            }
                        }
                        "open-launcher" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Handle window close - hide instead of quit
            if let Some(window) = app.get_webview_window("main") {
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window_clone.hide();
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            check_tool_status,
            update_tool,
            launch_tool,
            build_tool,
            set_tray_update_icon
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
