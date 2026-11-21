mod commands;
mod driver;
mod hidraw;
mod protocols;
mod safety;
mod state;
mod tray;

use state::ManagedState;
use std::io::Write;
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{Arc, Mutex};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("opensharkx3=debug".parse().unwrap()),
        )
        .init();

    let background = std::env::args().any(|a| a == "--background" || a == "--minimized");

    // Single instance: if another copy is already running, ask it to show its
    // window and exit immediately instead of opening a second copy.
    let config_dir = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let app_dir = config_dir.join("opensharkx3");
    let _ = std::fs::create_dir_all(&app_dir);
    let socket_path = app_dir.join("instance.sock");
    for _ in 0..20 {
        if let Ok(mut stream) = UnixStream::connect(&socket_path) {
            let _ = stream.write_all(b"show\n");
            std::process::exit(0);
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    let _ = std::fs::remove_file(&socket_path);
    let listener = UnixListener::bind(&socket_path).ok();
    let listener = Arc::new(Mutex::new(listener));

    tauri::Builder::default()
        .manage(ManagedState::new())
        .setup(move |app| {
            tray::create_tray(app.handle())?;

            // Listen for "show" requests from later launches and restore the
            // existing window instead of allowing a second instance.
            let app_handle = app.handle().clone();
            let listener = listener.clone();
            std::thread::spawn(move || {
                let Ok(guard) = listener.lock() else { return; };
                let Some(listener) = guard.as_ref() else { return; };
                for stream in listener.incoming() {
                    let _ = stream;
                    if let Some(win) = app_handle.get_webview_window("main") {
                        let _ = win.show();
                        let _ = win.set_focus();
                    }
                }
            });

            // DPI overlay window: transparent, always-on-top, hidden until needed.
            use tauri::{WebviewUrl, WebviewWindowBuilder};
            let _ = WebviewWindowBuilder::new(
                app,
                "dpi-overlay",
                WebviewUrl::App("overlay.html".into()),
            )
            .title("DPI")
            .inner_size(260.0, 90.0)
            .decorations(false)
            .transparent(true)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(false)
            .visible(false)
            .focused(false)
            .build();

            if background {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.hide();
                }
            }
            Ok(())
        })
        .on_window_event(|win, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = win.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::connect,
            commands::disconnect,
            commands::battery,
            commands::device_info,
            commands::discover_devices,
            commands::device_status,
            commands::descriptor_dump,
            commands::diagnose,
            commands::telemetry,
            commands::measure_polling_rate,
            commands::get_config,
            commands::save_config,
            commands::apply_config,
            commands::write_setting,
            commands::reset_config,
            commands::profiles_list,
            commands::profiles_save,
            commands::profiles_load,
            commands::profiles_delete,
            commands::autostart_status,
            commands::autostart_enable,
            commands::autostart_disable,
            commands::check_update,
            commands::apply_update,
            commands::show_dpi_overlay,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
