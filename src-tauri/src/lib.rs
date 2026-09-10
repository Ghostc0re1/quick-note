mod clipboard;
mod diagnostics;
mod keep_awake;
mod startup;
mod storage;
mod storage_path;

use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde::Serialize;
use storage::{Database, Note};
use tauri::{
    image::Image,
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, PhysicalPosition, PhysicalSize, RunEvent, WebviewWindow, WindowEvent,
};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

const CAPTURE_WINDOW: &str = "capture";
const HISTORY_WINDOW: &str = "history";
const ABOUT_WINDOW: &str = "about";
const TRAY_ICON_ID: &str = "quick-note";
const CAPTURE_ANIMATION_DURATION: Duration = Duration::from_millis(200);
const CAPTURE_ANIMATION_FRAMES: u32 = 12;
const CAPTURE_START_SIZE: u32 = 48;

struct AppState {
    database: Mutex<Database>,
    capture_animation: Arc<AtomicU64>,
    keep_awake: keep_awake::KeepAwake,
    keep_awake_menu: CheckMenuItem<tauri::Wry>,
    diagnostics: diagnostics::Diagnostics,
    diagnostics_menu: CheckMenuItem<tauri::Wry>,
    startup_menu: CheckMenuItem<tauri::Wry>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AboutInfo {
    version: String,
    diagnostics_enabled: bool,
}

const PRIVACY_POLICY_URL: &str = "https://github.com/Ghostc0re1/quick-note/blob/main/PRIVACY.md";
const SUPPORT_URL: &str = "https://github.com/Ghostc0re1/quick-note";

fn record_result<T>(
    diagnostics: &diagnostics::Diagnostics,
    event: &str,
    result: Result<T, String>,
) -> Result<T, String> {
    if result.is_err() {
        diagnostics.record(event);
    }
    result
}

#[tauri::command]
fn save_note(state: tauri::State<'_, AppState>, body: String) -> Result<Note, String> {
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("Could not read the system clock: {error}"))?
        .as_secs() as i64;
    let database = state
        .database
        .lock()
        .map_err(|_| "Scattered Thoughts database is unavailable.".to_owned())?;
    record_result(
        &state.diagnostics,
        "save-note-failed",
        database.save_note(&body, created_at),
    )
}

#[tauri::command]
fn list_notes(state: tauri::State<'_, AppState>, query: String) -> Result<Vec<Note>, String> {
    let database = state
        .database
        .lock()
        .map_err(|_| "Scattered Thoughts database is unavailable.".to_owned())?;
    record_result(
        &state.diagnostics,
        "list-notes-failed",
        database.list_notes(&query),
    )
}

#[tauri::command]
fn set_note_pinned(state: tauri::State<'_, AppState>, id: i64, pinned: bool) -> Result<(), String> {
    let database = state
        .database
        .lock()
        .map_err(|_| "Scattered Thoughts database is unavailable.".to_owned())?;
    database.set_note_pinned(id, pinned)
}

#[tauri::command]
fn delete_note(state: tauri::State<'_, AppState>, id: i64) -> Result<(), String> {
    let database = state
        .database
        .lock()
        .map_err(|_| "Scattered Thoughts database is unavailable.".to_owned())?;
    database.delete_note(id)
}

#[tauri::command]
fn copy_note(state: tauri::State<'_, AppState>, id: i64) -> Result<(), String> {
    let body = state
        .database
        .lock()
        .map_err(|_| "Scattered Thoughts database is unavailable.".to_owned())?
        .note_body(id)?;
    clipboard::write_text(&body)
}

#[tauri::command]
fn about_info(state: tauri::State<'_, AppState>) -> AboutInfo {
    AboutInfo {
        version: env!("CARGO_PKG_VERSION").to_owned(),
        diagnostics_enabled: state.diagnostics.is_enabled(),
    }
}

#[tauri::command]
fn hide_capture(app: AppHandle) -> Result<(), String> {
    hide_window(&app, CAPTURE_WINDOW)
}

async fn choose_save_path(
    app: AppHandle,
    title: &'static str,
    file_name: &'static str,
    extension: &'static str,
) -> Result<Option<PathBuf>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .set_title(title)
            .set_file_name(file_name)
            .add_filter(file_name, &[extension])
            .blocking_save_file()
            .map(|file| {
                file.into_path()
                    .map_err(|error| format!("Could not use the selected file: {error}"))
            })
            .transpose()
    })
    .await
    .map_err(|error| format!("Could not show the save dialog: {error}"))?
}

#[tauri::command]
async fn export_notes(app: AppHandle, state: tauri::State<'_, AppState>) -> Result<bool, String> {
    let markdown = state
        .database
        .lock()
        .map_err(|_| "Scattered Thoughts database is unavailable.".to_owned())?
        .export_markdown()?;
    let Some(destination) = choose_save_path(
        app,
        "Export Scattered Thoughts notes",
        "scattered-thoughts-notes.md",
        "md",
    )
    .await?
    else {
        return Ok(false);
    };
    fs::write(destination, markdown).map_err(|error| format!("Could not export notes: {error}"))?;
    Ok(true)
}

#[tauri::command]
async fn backup_database(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<bool, String> {
    let Some(destination) = choose_save_path(
        app,
        "Back up Scattered Thoughts database",
        "scattered-thoughts-backup.sqlite3",
        "sqlite3",
    )
    .await?
    else {
        return Ok(false);
    };
    state
        .database
        .lock()
        .map_err(|_| "Scattered Thoughts database is unavailable.".to_owned())?
        .backup_to(&destination)?;
    Ok(true)
}

#[tauri::command]
fn open_about_link(kind: String) -> Result<(), String> {
    let url = match kind.as_str() {
        "privacy" => PRIVACY_POLICY_URL,
        "support" => SUPPORT_URL,
        _ => return Err("That link is unavailable.".to_owned()),
    };
    Command::new("explorer")
        .arg(url)
        .spawn()
        .map_err(|error| format!("Could not open the link: {error}"))?;
    Ok(())
}

#[tauri::command]
fn open_diagnostics_folder(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let directory = state.diagnostics.directory();
    fs::create_dir_all(directory)
        .map_err(|error| format!("Could not create diagnostics directory: {error}"))?;
    Command::new("explorer")
        .arg(directory)
        .spawn()
        .map_err(|error| format!("Could not open diagnostics folder: {error}"))?;
    Ok(())
}

fn window(app: &AppHandle, label: &str) -> Result<WebviewWindow, String> {
    app.get_webview_window(label)
        .ok_or_else(|| format!("Scattered Thoughts {label} window is unavailable."))
}

fn hide_window(app: &AppHandle, label: &str) -> Result<(), String> {
    if label == CAPTURE_WINDOW {
        app.state::<AppState>()
            .capture_animation
            .fetch_add(1, Ordering::Relaxed);
    }
    window(app, label)?
        .hide()
        .map_err(|error| error.to_string())
}

fn show_window(app: &AppHandle, label: &str) -> Result<(), String> {
    for other_label in [CAPTURE_WINDOW, HISTORY_WINDOW, ABOUT_WINDOW] {
        if other_label != label {
            hide_window(app, other_label)?;
        }
    }
    let target = window(app, label)?;
    target.center().map_err(|error| error.to_string())?;
    target.show().map_err(|error| error.to_string())?;
    target.set_focus().map_err(|error| error.to_string())
}

fn show_capture(app: &AppHandle) -> Result<(), String> {
    hide_window(app, HISTORY_WINDOW)?;
    hide_window(app, ABOUT_WINDOW)?;
    let target = window(app, CAPTURE_WINDOW)?;
    target.center().map_err(|error| error.to_string())?;
    let final_position = target.outer_position().map_err(|error| error.to_string())?;
    let final_size = target.outer_size().map_err(|error| error.to_string())?;
    let scale_factor = target.scale_factor().map_err(|error| error.to_string())?;

    let Some(start_position) = tray_center(app, scale_factor) else {
        target.show().map_err(|error| error.to_string())?;
        return target.set_focus().map_err(|error| error.to_string());
    };

    let start_size = PhysicalSize::new(CAPTURE_START_SIZE, CAPTURE_START_SIZE);
    let start_position = PhysicalPosition::new(
        start_position.x - (CAPTURE_START_SIZE as i32 / 2),
        start_position.y - (CAPTURE_START_SIZE as i32 / 2),
    );
    let animation = app.state::<AppState>().capture_animation.clone();
    let animation_id = animation.fetch_add(1, Ordering::Relaxed) + 1;

    target
        .set_size(start_size)
        .map_err(|error| error.to_string())?;
    target
        .set_position(start_position)
        .map_err(|error| error.to_string())?;
    target.show().map_err(|error| error.to_string())?;
    target.set_focus().map_err(|error| error.to_string())?;

    thread::spawn(move || {
        let frame_duration = CAPTURE_ANIMATION_DURATION / CAPTURE_ANIMATION_FRAMES;
        for frame in 1..=CAPTURE_ANIMATION_FRAMES {
            thread::sleep(frame_duration);
            if animation.load(Ordering::Relaxed) != animation_id {
                return;
            }

            let progress = ease_out_cubic(frame as f64 / CAPTURE_ANIMATION_FRAMES as f64);
            let _ = target.set_size(PhysicalSize::new(
                interpolate_u32(start_size.width, final_size.width, progress),
                interpolate_u32(start_size.height, final_size.height, progress),
            ));
            let _ = target.set_position(PhysicalPosition::new(
                interpolate_i32(start_position.x, final_position.x, progress),
                interpolate_i32(start_position.y, final_position.y, progress),
            ));
        }
    });

    Ok(())
}

fn show_history(app: &AppHandle) -> Result<(), String> {
    show_window(app, HISTORY_WINDOW)
}

fn show_about(app: &AppHandle) -> Result<(), String> {
    show_window(app, ABOUT_WINDOW)
}

fn hide_visible_windows(app: &AppHandle) {
    for label in [CAPTURE_WINDOW, HISTORY_WINDOW, ABOUT_WINDOW] {
        if let Some(target) = app.get_webview_window(label) {
            let _ = target.hide();
        }
    }
}

fn tray_center(app: &AppHandle, scale_factor: f64) -> Option<PhysicalPosition<i32>> {
    let rect = app.tray_by_id(TRAY_ICON_ID)?.rect().ok()??;
    let position = rect.position.to_physical::<i32>(scale_factor);
    let size = rect.size.to_physical::<u32>(scale_factor);
    Some(PhysicalPosition::new(
        position.x + (size.width as i32 / 2),
        position.y + (size.height as i32 / 2),
    ))
}

fn ease_out_cubic(progress: f64) -> f64 {
    1.0 - (1.0 - progress).powi(3)
}

fn interpolate_i32(start: i32, end: i32, progress: f64) -> i32 {
    (start as f64 + (end - start) as f64 * progress).round() as i32
}

fn interpolate_u32(start: u32, end: u32, progress: f64) -> u32 {
    (start as f64 + (end - start) as f64 * progress).round() as u32
}

fn tray_icon() -> Image<'static> {
    let width = 32;
    let height = 32;
    let mut pixels = vec![0_u8; width * height * 4];

    for y in 0..height {
        for x in 0..width {
            let offset = (y * width + x) * 4;
            let inside_note = (5..27).contains(&x) && (3..29).contains(&y);
            let folded_corner = x > 20 && y < 10 && x + y > 30;
            let line =
                inside_note && !folded_corner && matches!(y, 12 | 17 | 22) && (9..24).contains(&x);

            if inside_note && !folded_corner {
                pixels[offset..offset + 4].copy_from_slice(&[35, 92, 178, 255]);
            }
            if line {
                pixels[offset..offset + 4].copy_from_slice(&[255, 255, 255, 255]);
            }
        }
    }

    Image::new_owned(pixels, width as u32, height as u32)
}

fn sync_startup_menu(app: &AppHandle) {
    let item = &app.state::<AppState>().startup_menu;
    let (checked, enabled, label) = match startup::status() {
        startup::StartupStatus::Enabled => (true, true, "Launch at sign-in"),
        startup::StartupStatus::Disabled
        | startup::StartupStatus::DisabledByUser
        | startup::StartupStatus::DisabledByPolicy => (false, true, "Launch at sign-in"),
        startup::StartupStatus::Unavailable => (false, false, "Launch at sign-in (Store edition)"),
    };
    let _ = item.set_checked(checked);
    let _ = item.set_enabled(enabled);
    let _ = item.set_text(label);
}

fn toggle_startup(app: &AppHandle) {
    if let Err(error) = startup::toggle() {
        app.dialog()
            .message(error)
            .title("Scattered Thoughts startup")
            .kind(MessageDialogKind::Info)
            .show(|_| {});
    }
    sync_startup_menu(app);
}

fn sync_keep_awake_menu(app: &AppHandle) {
    let state = app.state::<AppState>();
    let _ = state
        .keep_awake_menu
        .set_checked(state.keep_awake.is_enabled());
}

fn toggle_keep_awake(app: &AppHandle) {
    let state = app.state::<AppState>();
    let requested = !state.keep_awake.is_enabled();
    if let Err(error) = state.keep_awake.set_enabled(requested) {
        app.dialog()
            .message(error)
            .title("Keep PC awake")
            .kind(MessageDialogKind::Error)
            .show(|_| {});
    }
    sync_keep_awake_menu(app);
}
fn sync_diagnostics_menu(app: &AppHandle) {
    let state = app.state::<AppState>();
    let _ = state
        .diagnostics_menu
        .set_checked(state.diagnostics.is_enabled());
}

fn toggle_diagnostics(app: &AppHandle) {
    let state = app.state::<AppState>();
    let requested = !state.diagnostics.is_enabled();
    let result = state
        .database
        .lock()
        .map_err(|_| "Scattered Thoughts database is unavailable.".to_owned())
        .and_then(|database| database.set_diagnostics_enabled(requested));
    match result {
        Ok(()) => state.diagnostics.set_enabled(requested),
        Err(error) => {
            app.dialog()
                .message(error)
                .title("Diagnostic logging")
                .kind(MessageDialogKind::Error)
                .show(|_| {});
        }
    }
    sync_diagnostics_menu(app);
}

fn build_tray(
    app: &tauri::App,
) -> tauri::Result<(
    CheckMenuItem<tauri::Wry>,
    CheckMenuItem<tauri::Wry>,
    CheckMenuItem<tauri::Wry>,
)> {
    let show = MenuItem::with_id(app, "show", "Show Scattered Thoughts", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;
    let history = MenuItem::with_id(app, "history", "Recent Notes", true, None::<&str>)?;
    let keep_awake = CheckMenuItem::with_id(
        app,
        "keep-awake",
        "Keep PC awake",
        true,
        false,
        None::<&str>,
    )?;
    let diagnostics = CheckMenuItem::with_id(
        app,
        "diagnostics",
        "Enable diagnostic logging",
        true,
        false,
        None::<&str>,
    )?;
    let startup = CheckMenuItem::with_id(
        app,
        "startup",
        "Launch at sign-in (Store edition)",
        false,
        false,
        None::<&str>,
    )?;
    let about = MenuItem::with_id(app, "about", "About Scattered Thoughts", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let first_separator = PredefinedMenuItem::separator(app)?;
    let second_separator = PredefinedMenuItem::separator(app)?;
    let third_separator = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(
        app,
        &[
            &show,
            &hide,
            &first_separator,
            &history,
            &second_separator,
            &keep_awake,
            &diagnostics,
            &startup,
            &third_separator,
            &about,
            &quit,
        ],
    )?;

    TrayIconBuilder::with_id(TRAY_ICON_ID)
        .icon(tray_icon())
        .tooltip("Scattered Thoughts")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .build(app)?;
    Ok((keep_awake, diagnostics, startup))
}

fn register_shortcut(app: &tauri::App) {
    let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::Space);
    if app.global_shortcut().register(shortcut).is_err() {
        app.dialog()
            .message(
                "Ctrl+Alt+Space is already in use. Use the Scattered Thoughts tray icon instead.",
            )
            .title("Scattered Thoughts shortcut unavailable")
            .kind(MessageDialogKind::Warning)
            .show(|_| {});
    }
}

pub fn run() {
    let app = tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        let _ = show_capture(app);
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let unpackaged_data_dir = app.path().app_data_dir()?;
            let app_data_dir =
                storage_path::data_directory(unpackaged_data_dir).map_err(std::io::Error::other)?;
            let database = match Database::open(&app_data_dir) {
                Ok(database) => database,
                Err(error) => {
                    app.dialog()
                        .message(&error)
                        .title("Scattered Thoughts could not start")
                        .kind(MessageDialogKind::Error)
                        .show(|_| {});
                    return Err(std::io::Error::other(error).into());
                }
            };
            let diagnostics_enabled = database
                .diagnostics_enabled()
                .map_err(std::io::Error::other)?;
            let diagnostics = diagnostics::Diagnostics::new(
                app_data_dir.join("diagnostics"),
                diagnostics_enabled,
            );
            let (keep_awake_menu, diagnostics_menu, startup_menu) = build_tray(app)?;
            app.manage(AppState {
                database: Mutex::new(database),
                capture_animation: Arc::new(AtomicU64::new(0)),
                keep_awake: keep_awake::KeepAwake::new(),
                keep_awake_menu,
                diagnostics,
                diagnostics_menu,
                startup_menu,
            });
            sync_keep_awake_menu(app.handle());
            sync_diagnostics_menu(app.handle());
            sync_startup_menu(app.handle());
            register_shortcut(app);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            save_note,
            list_notes,
            set_note_pinned,
            delete_note,
            copy_note,
            export_notes,
            backup_database,
            about_info,
            open_about_link,
            open_diagnostics_folder,
            hide_capture,
        ])
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => {
                let _ = show_capture(app);
            }
            "hide" => hide_visible_windows(app),
            "history" => {
                let _ = show_history(app);
            }
            "startup" => toggle_startup(app),
            "keep-awake" => toggle_keep_awake(app),
            "diagnostics" => toggle_diagnostics(app),
            "about" => {
                let _ = show_about(app);
            }
            "quit" => {
                let _ = app.state::<AppState>().keep_awake.set_enabled(false);
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|app, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let capture_is_visible = app
                    .get_webview_window(CAPTURE_WINDOW)
                    .and_then(|window| window.is_visible().ok())
                    .unwrap_or(false);
                if capture_is_visible {
                    let _ = hide_window(app, CAPTURE_WINDOW);
                } else {
                    let _ = show_capture(app);
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building Scattered Thoughts");

    app.run(|app, event| {
        if let RunEvent::WindowEvent {
            label,
            event: WindowEvent::CloseRequested { api, .. },
            ..
        } = event
        {
            if label == CAPTURE_WINDOW || label == HISTORY_WINDOW || label == ABOUT_WINDOW {
                api.prevent_close();
                let _ = hide_window(app, &label);
            }
        }
    });
}
