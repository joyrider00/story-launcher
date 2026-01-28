use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;

use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use tar::Archive;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, Runtime,
};

// Tool configuration
const RESOLVE_SYNC_REPO: &str = "joyrider00/spellbook-resolve-sync";
const RESOLVE_SYNC_APP_NAME: &str = "Spellbook Resolve Sync.app";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolStatus {
    pub installed: bool,
    pub installed_version: Option<String>,
    pub latest_version: Option<String>,
    pub has_update: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionResult {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct ToolsConfig {
    #[serde(default)]
    tools: HashMap<String, String>, // tool_id -> version
}

// GitHub API response types
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

// Global state
pub struct AppState {
    pub has_updates: Mutex<bool>,
}

fn get_tools_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_default();
    home.join(".story-tools")
}

fn get_apps_dir() -> PathBuf {
    get_tools_dir().join("apps")
}

fn get_config_path() -> PathBuf {
    get_tools_dir().join("config.json")
}

fn ensure_dirs() -> io::Result<()> {
    fs::create_dir_all(get_apps_dir())?;
    Ok(())
}

fn load_config() -> ToolsConfig {
    let config_path = get_config_path();
    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str(&content) {
                return config;
            }
        }
    }
    ToolsConfig::default()
}

fn save_config(config: &ToolsConfig) -> io::Result<()> {
    ensure_dirs()?;
    let config_path = get_config_path();
    let content = serde_json::to_string_pretty(config)?;
    fs::write(config_path, content)?;
    Ok(())
}

fn get_latest_release(repo: &str) -> Result<GitHubRelease, String> {
    let url = format!("https://api.github.com/repos/{}/releases/latest", repo);

    let client = reqwest::blocking::Client::builder()
        .user_agent("Story-Launcher/1.0")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(&url)
        .send()
        .map_err(|e| format!("Failed to fetch release info: {}", e))?;

    if response.status() == 403 {
        return Err("GitHub API rate limit exceeded. Please try again later.".to_string());
    }

    if response.status() == 404 {
        return Err("No releases found for this repository.".to_string());
    }

    if !response.status().is_success() {
        return Err(format!("GitHub API error: {}", response.status()));
    }

    response
        .json::<GitHubRelease>()
        .map_err(|e| format!("Failed to parse release info: {}", e))
}

fn find_app_asset(release: &GitHubRelease) -> Option<&GitHubAsset> {
    // Look for .app.tar.gz first (preferred), then .app.zip, then .dmg
    release
        .assets
        .iter()
        .find(|a| a.name.ends_with(".app.tar.gz"))
        .or_else(|| release.assets.iter().find(|a| a.name.ends_with(".app.zip")))
        .or_else(|| release.assets.iter().find(|a| a.name.ends_with(".dmg")))
}

fn download_file(url: &str, dest: &PathBuf) -> Result<(), String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("Story-Launcher/1.0")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(url)
        .send()
        .map_err(|e| format!("Failed to download: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed: {}", response.status()));
    }

    let bytes = response
        .bytes()
        .map_err(|e| format!("Failed to read download: {}", e))?;

    let mut file = File::create(dest).map_err(|e| format!("Failed to create file: {}", e))?;

    file.write_all(&bytes)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}

fn extract_tar_gz(archive_path: &PathBuf, dest_dir: &PathBuf) -> Result<(), String> {
    let file = File::open(archive_path).map_err(|e| format!("Failed to open archive: {}", e))?;

    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);

    archive
        .unpack(dest_dir)
        .map_err(|e| format!("Failed to extract archive: {}", e))?;

    Ok(())
}

fn extract_zip(archive_path: &PathBuf, dest_dir: &PathBuf) -> Result<(), String> {
    let file = File::open(archive_path).map_err(|e| format!("Failed to open archive: {}", e))?;

    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Failed to read zip: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read zip entry: {}", e))?;

        let outpath = dest_dir.join(file.name());

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }
            let mut outfile =
                File::create(&outpath).map_err(|e| format!("Failed to create file: {}", e))?;
            io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to write file: {}", e))?;
        }

        // Set permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).ok();
            }
        }
    }

    Ok(())
}

fn get_app_path(app_name: &str) -> PathBuf {
    get_apps_dir().join(app_name)
}

fn is_tool_installed(tool_id: &str) -> bool {
    let config = load_config();
    if !config.tools.contains_key(tool_id) {
        return false;
    }

    // Also verify the app actually exists
    let app_path = match tool_id {
        "resolve-sync" => get_app_path(RESOLVE_SYNC_APP_NAME),
        _ => return false,
    };

    app_path.exists()
}

fn get_installed_version(tool_id: &str) -> Option<String> {
    let config = load_config();
    config.tools.get(tool_id).cloned()
}

#[tauri::command]
fn check_tool_status(tool_id: String) -> ToolStatus {
    let repo = match tool_id.as_str() {
        "resolve-sync" => RESOLVE_SYNC_REPO,
        _ => {
            return ToolStatus {
                installed: false,
                installed_version: None,
                latest_version: None,
                has_update: false,
                error: Some("Unknown tool".to_string()),
            }
        }
    };

    let installed = is_tool_installed(&tool_id);
    let installed_version = get_installed_version(&tool_id);

    // Fetch latest release from GitHub
    match get_latest_release(repo) {
        Ok(release) => {
            let latest_version = release.tag_name.trim_start_matches('v').to_string();
            let has_update = installed
                && installed_version
                    .as_ref()
                    .map(|v| v != &latest_version)
                    .unwrap_or(false);

            ToolStatus {
                installed,
                installed_version,
                latest_version: Some(latest_version),
                has_update,
                error: None,
            }
        }
        Err(e) => ToolStatus {
            installed,
            installed_version,
            latest_version: None,
            has_update: false,
            error: Some(e),
        },
    }
}

#[tauri::command]
fn install_tool(tool_id: String) -> ActionResult {
    let (repo, app_name) = match tool_id.as_str() {
        "resolve-sync" => (RESOLVE_SYNC_REPO, RESOLVE_SYNC_APP_NAME),
        _ => {
            return ActionResult {
                success: false,
                message: "Unknown tool".to_string(),
            }
        }
    };

    // Ensure directories exist
    if let Err(e) = ensure_dirs() {
        return ActionResult {
            success: false,
            message: format!("Failed to create directories: {}", e),
        };
    }

    // Get latest release
    let release = match get_latest_release(repo) {
        Ok(r) => r,
        Err(e) => {
            return ActionResult {
                success: false,
                message: e,
            }
        }
    };

    // Find downloadable asset
    let asset = match find_app_asset(&release) {
        Some(a) => a,
        None => {
            return ActionResult {
                success: false,
                message: "No compatible download found in release".to_string(),
            }
        }
    };

    // Download to temp file
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(&asset.name);

    if let Err(e) = download_file(&asset.browser_download_url, &temp_file) {
        return ActionResult {
            success: false,
            message: e,
        };
    }

    // Remove existing app if present
    let app_path = get_app_path(app_name);
    if app_path.exists() {
        if let Err(e) = fs::remove_dir_all(&app_path) {
            return ActionResult {
                success: false,
                message: format!("Failed to remove existing app: {}", e),
            };
        }
    }

    // Extract based on file type
    let apps_dir = get_apps_dir();
    let result = if asset.name.ends_with(".tar.gz") {
        extract_tar_gz(&temp_file, &apps_dir)
    } else if asset.name.ends_with(".zip") {
        extract_zip(&temp_file, &apps_dir)
    } else if asset.name.ends_with(".dmg") {
        // For DMG, we need to mount, copy, and unmount
        extract_from_dmg(&temp_file, &apps_dir, app_name)
    } else {
        Err("Unsupported archive format".to_string())
    };

    // Clean up temp file
    let _ = fs::remove_file(&temp_file);

    if let Err(e) = result {
        return ActionResult {
            success: false,
            message: e,
        };
    }

    // Remove quarantine attribute
    let _ = Command::new("xattr")
        .args(["-cr", app_path.to_str().unwrap_or("")])
        .output();

    // Update config
    let mut config = load_config();
    let version = release.tag_name.trim_start_matches('v').to_string();
    config.tools.insert(tool_id.clone(), version.clone());

    if let Err(e) = save_config(&config) {
        return ActionResult {
            success: false,
            message: format!("Failed to save config: {}", e),
        };
    }

    ActionResult {
        success: true,
        message: format!("Installed version {}", version),
    }
}

fn extract_from_dmg(dmg_path: &PathBuf, dest_dir: &PathBuf, app_name: &str) -> Result<(), String> {
    // Mount DMG
    let output = Command::new("hdiutil")
        .args(["attach", dmg_path.to_str().unwrap(), "-nobrowse", "-quiet"])
        .output()
        .map_err(|e| format!("Failed to mount DMG: {}", e))?;

    if !output.status.success() {
        return Err("Failed to mount DMG".to_string());
    }

    // Find mount point
    let output = Command::new("hdiutil")
        .args(["info", "-plist"])
        .output()
        .map_err(|e| format!("Failed to get mount info: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mount_point = stdout
        .lines()
        .skip_while(|l| !l.contains("/Volumes/"))
        .next()
        .and_then(|l| l.split("<string>").nth(1))
        .and_then(|l| l.split("</string>").next())
        .ok_or("Failed to find mount point")?
        .to_string();

    // Copy app
    let src = PathBuf::from(&mount_point).join(app_name);
    let dest = dest_dir.join(app_name);

    let copy_result = Command::new("cp")
        .args(["-R", src.to_str().unwrap(), dest.to_str().unwrap()])
        .output();

    // Unmount DMG
    let _ = Command::new("hdiutil")
        .args(["detach", &mount_point, "-quiet"])
        .output();

    copy_result
        .map_err(|e| format!("Failed to copy app: {}", e))
        .and_then(|o| {
            if o.status.success() {
                Ok(())
            } else {
                Err("Failed to copy app from DMG".to_string())
            }
        })
}

#[tauri::command]
fn update_tool(tool_id: String) -> ActionResult {
    // Update is the same as install - it will replace the existing version
    install_tool(tool_id)
}

#[tauri::command]
fn launch_tool(tool_id: String) -> ActionResult {
    let app_name = match tool_id.as_str() {
        "resolve-sync" => RESOLVE_SYNC_APP_NAME,
        _ => {
            return ActionResult {
                success: false,
                message: "Unknown tool".to_string(),
            }
        }
    };

    let app_path = get_app_path(app_name);

    if !app_path.exists() {
        return ActionResult {
            success: false,
            message: "App not installed".to_string(),
        };
    }

    match Command::new("open").arg(&app_path).spawn() {
        Ok(_) => ActionResult {
            success: true,
            message: "Launched app".to_string(),
        },
        Err(e) => ActionResult {
            success: false,
            message: format!("Failed to launch: {}", e),
        },
    }
}

#[tauri::command]
fn get_installed_tools() -> Vec<String> {
    let config = load_config();
    config
        .tools
        .keys()
        .filter(|tool_id| is_tool_installed(tool_id))
        .cloned()
        .collect()
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

fn create_tray_menu<R: Runtime>(
    app: &tauri::AppHandle<R>,
    installed_tools: &[String],
) -> tauri::Result<Menu<R>> {
    let mut items: Vec<Box<dyn tauri::menu::IsMenuItem<R>>> = Vec::new();

    // Add installed tools
    if installed_tools.contains(&"resolve-sync".to_string()) {
        items.push(Box::new(MenuItem::with_id(
            app,
            "resolve-sync",
            "Resolve Sync Script",
            true,
            None::<&str>,
        )?));
    }

    // Always show web apps
    items.push(Box::new(MenuItem::with_id(
        app,
        "spellbook",
        "Spellbook",
        true,
        None::<&str>,
    )?));
    items.push(Box::new(MenuItem::with_id(
        app,
        "portal",
        "Story Portal",
        true,
        None::<&str>,
    )?));

    items.push(Box::new(PredefinedMenuItem::separator(app)?));
    items.push(Box::new(MenuItem::with_id(
        app,
        "check-updates",
        "Check for Updates",
        true,
        None::<&str>,
    )?));
    items.push(Box::new(MenuItem::with_id(
        app,
        "open-launcher",
        "Open Story Launcher",
        true,
        None::<&str>,
    )?));
    items.push(Box::new(PredefinedMenuItem::separator(app)?));
    items.push(Box::new(MenuItem::with_id(
        app,
        "quit",
        "Quit Story Launcher",
        true,
        None::<&str>,
    )?));

    // Build menu from refs
    let item_refs: Vec<&dyn tauri::menu::IsMenuItem<R>> = items.iter().map(|b| b.as_ref()).collect();
    Menu::with_items(app, &item_refs)
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

            // Get installed tools for tray menu
            let installed_tools = get_installed_tools();

            // Create tray icon
            let tray_icon = Image::from_bytes(include_bytes!("../icons/tray-icon.png"))?;
            let menu = create_tray_menu(&handle, &installed_tools)?;

            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(tray_icon)
                .menu(&menu)
                .tooltip("Story Launcher")
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "resolve-sync" => {
                        let _ = launch_tool("resolve-sync".to_string());
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
            install_tool,
            update_tool,
            launch_tool,
            get_installed_tools,
            set_tray_update_icon
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
