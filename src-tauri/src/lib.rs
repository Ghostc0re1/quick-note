mod startup;
mod storage;
mod storage_path;

use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

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
const TRAY_ICON_ID: &str = "quick-note";
const CAPTURE_ANIMATION_DURATION: Duration = Duration::from_millis(200);
const CAPTURE_ANIMATION_FRAMES: u32 = 12;
const CAPTURE_START_SIZE: u32 = 48;

struct AppState {
    database: Mutex<Database>,
    capture_animation: Arc<AtomicU64>,
    startup_menu: CheckMenuItem<tauri::Wry>,
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
        .map_err(|_| "Quick Note database is unavailable.".to_owned())?;
    database.save_note(&body, created_at)
}

#[tauri::command]
fn list_recent_notes(state: tauri::State<'_, AppState>) -> Result<Vec<Note>, String> {
    let database = state
        .database
        .lock()
        .map_err(|_| "Quick Note database is unavailable.".to_owned())?;
    database.list_recent_notes()
}

#[tauri::command]
fn hide_capture(app: AppHandle) -> Result<(), String> {
    hide_window(&app, CAPTURE_WINDOW)
}

fn window(app: &AppHandle, label: &str) -> Result<WebviewWindow, String> {
    app.get_webview_window(label)
        .ok_or_else(|| format!("Quick Note {label} window is unavailable."))
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

fn show_window(app: &AppHandle, label: &str, other_label: &str) -> Result<(), String> {
    hide_window(app, other_label)?;
    let target = window(app, label)?;
    target.center().map_err(|error| error.to_string())?;
    target.show().map_err(|error| error.to_string())?;
    target.set_focus().map_err(|error| error.to_string())
}

fn show_capture(app: &AppHandle) -> Result<(), String> {
    hide_window(app, HISTORY_WINDOW)?;
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
    show_window(app, HISTORY_WINDOW, CAPTURE_WINDOW)
}

fn hide_visible_windows(app: &AppHandle) {
    for label in [CAPTURE_WINDOW, HISTORY_WINDOW] {
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
            .title("Quick Note startup")
            .kind(MessageDialogKind::Info)
            .show(|_| {});
    }
    sync_startup_menu(app);
}

fn build_tray(app: &tauri::App) -> tauri::Result<CheckMenuItem<tauri::Wry>> {
    let show = MenuItem::with_id(app, "show", "Show Quick Note", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide Quick Note", true, None::<&str>)?;
    let history = MenuItem::with_id(app, "history", "Recent Notes", true, None::<&str>)?;
    let startup = CheckMenuItem::with_id(
        app,
        "startup",
        "Launch at sign-in (Store edition)",
        false,
        false,
        None::<&str>,
    )?;
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
            &startup,
            &third_separator,
            &quit,
        ],
    )?;

    TrayIconBuilder::with_id(TRAY_ICON_ID)
        .icon(tray_icon())
        .tooltip("Quick Note")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .build(app)?;
    Ok(startup)
}

fn register_shortcut(app: &tauri::App) {
    let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::Space);
    if app.global_shortcut().register(shortcut).is_err() {
        app.dialog()
            .message("Ctrl+Alt+Space is already in use. Use the Quick Note tray icon instead.")
            .title("Quick Note shortcut unavailable")
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
                        .title("Quick Note could not start")
                        .kind(MessageDialogKind::Error)
                        .show(|_| {});
                    return Err(std::io::Error::other(error).into());
                }
            };
            let startup_menu = build_tray(app)?;
            app.manage(AppState {
                database: Mutex::new(database),
                capture_animation: Arc::new(AtomicU64::new(0)),
                startup_menu,
            });
            sync_startup_menu(app.handle());
            register_shortcut(app);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            save_note,
            list_recent_notes,
            hide_capture
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
            "quit" => app.exit(0),
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
        .expect("error while building Quick Note");

    app.run(|app, event| {
        if let RunEvent::WindowEvent {
            label,
            event: WindowEvent::CloseRequested { api, .. },
            ..
        } = event
        {
            if label == CAPTURE_WINDOW || label == HISTORY_WINDOW {
                api.prevent_close();
                let _ = hide_window(app, &label);
            }
        }
    });
}
