use std::{sync::mpsc, thread};

use crate::{AppState, Status};

use tauri::{AppHandle, Manager};
use tempfile::NamedTempFile;

mod utils;
use utils::{get_random_id, start_frame_capture};

mod encoder;
use encoder::preview_encoder_thread_handler;

#[tauri::command]
pub async fn start_recording(app_handle: AppHandle) {
    let app_handle_clone = app_handle.clone();
    start_frame_capture(app_handle_clone).await;

    let state = app_handle.state::<AppState>();

    // Start capturing frames
    println!("Capturing frames...");
    let mut frames = state.frames.lock().await;

    // Reset frames to empty array to allow user to
    // record multiple gifs without restarting the app
    *frames = Vec::new();

    // Create file in temp directory
    let preview_file = format!("HM-{}.mp4", get_random_id());
    let mut preview_path = state.preview_path.lock().await;
    *preview_path = Some(
        NamedTempFile::new()
            .unwrap()
            .into_temp_path()
            .with_file_name(&preview_file),
    );

    let mut recorder = state.recorder.lock().await;
    let [output_width, output_height] = (*recorder).as_mut().unwrap().get_output_frame_size();
    drop(recorder);

    let output_path = preview_path.as_ref().unwrap().to_str().unwrap().to_string();

    let (tx, rx) = mpsc::channel();

    // Spawn a processing thread
    let preview_encoding_thread = thread::spawn(move || {
        preview_encoder_thread_handler(
            rx,
            output_width as usize,
            output_height as usize,
            output_path,
        );
    });

    loop {
        let mut recorder = state.recorder.lock().await;

        if recorder.is_none() {
            println!("Exiting encoding loop");
            break;
        }

        let frame = (*recorder)
            .as_mut()
            .unwrap()
            .get_next_frame()
            .expect("Error");

        let frame_for_preview = frame.clone();
        tx.send(frame_for_preview).unwrap();

        (*frames).push(frame);
        drop(recorder);
    }

    // drop the sender to close the channel
    drop(tx);
    // wait for the encoding thread to finish
    preview_encoding_thread
        .join()
        .expect("Processing thread panicked.");

    println!("Creating Editor Window");
    println!("Preview path: {:?}", preview_path);
    println!("Preview dimensions: {}x{}", output_width, output_height);

    // initialise the editor with the file path of encoded video
    crate::editor::init_editor(
        &app_handle,
        preview_path.as_ref().unwrap().to_str().unwrap().to_string(),
    );
}

#[tauri::command]
pub async fn stop_recording(app_handle: AppHandle) {
    // Hide cropper and toolbar
    crate::cropper::toggle_cropper(&app_handle);
    crate::toolbar::toggle_toolbar(&app_handle);

    // Stop capturing frames and drop recorder
    let state = app_handle.state::<AppState>();
    let mut recorder = state.recorder.lock().await;
    (*recorder).as_mut().unwrap().stop_capture();
    recorder.take();
    drop(recorder);

    // Update app state to editing
    let mut status = state.status.lock().await;
    *status = Status::Editing;
    drop(status);
}