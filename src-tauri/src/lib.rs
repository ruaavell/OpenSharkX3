mod commands;
mod driver;
mod hidraw;
mod protocols;
mod safety;
mod state;
mod tray;

use state::ManagedState;
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

    tauri::Builder::default()
        .manage(ManagedState::new())
        .setup(move |app| {
            tray::create_tray(app.handle())?;

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
