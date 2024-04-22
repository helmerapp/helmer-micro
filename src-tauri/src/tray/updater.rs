use tauri::{AppHandle, Result};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_updater::UpdaterExt;

pub fn check_for_update(app_handle: AppHandle) -> Result<()> {
    tauri::async_runtime::spawn(async move {
        let updater = app_handle.updater();
        match updater {
            Ok(updater) => {
                let response = updater.check().await;
                match response {
                    Ok(update_option) => {
                        if let Some(update) = update_option {
                            let update_str = format!(
                                "Current Version: {}\nLatest Version: {}",
                                update.current_version, update.version
                            );

                            println!("{}", update_str);

                            app_handle
                                .dialog()
                                .message(update_str)
                                .title("A new version of Helmer Micro is out!")
                                .ok_button_label("Install")
                                .cancel_button_label("Cancel")
                                .show(|result| match result {
                                    true => {
                                        // TODO: install update
                                        // update.download_and_install(None, None);
                                    }
                                    _ => {}
                                });
                            println!(
                                "update available:\n\tdownload url: {}\n\tsignature: {}",
                                update.download_url, update.signature
                            );
                        } else {
                            app_handle
                                .dialog()
                                .message("You're on the latest version!")
                                .ok_button_label("Okay")
                                .show(|_| {});
                        }
                    }
                    Err(e) => {
                        eprintln!("failed to check for updates: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("failed to build updater: {}", e);
            }
        }
    });
    Ok(())
}