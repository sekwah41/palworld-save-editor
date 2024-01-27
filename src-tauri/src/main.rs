// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::ffi::OsString;
use tauri::{FileDropEvent, Manager, RunEvent};
use std::fs::File;
use std::io::{self, Read, Write};
use flate2::read::ZlibDecoder;
use std::process::Command;
use uesave::{Property, Save, StructType, Types};

const PALWORLD_TYPES: [(&str, &str); 6] = [
    (".worldSaveData.CharacterSaveParameterMap.Key", "Struct"),
    (".worldSaveData.FoliageGridSaveDataMap.Key", "Struct"),
    (".worldSaveData.FoliageGridSaveDataMap.ModelMap.InstanceDataMap.Key", "Struct"),
    (".worldSaveData.MapObjectSpawnerInStageSaveData.Key", "Struct"),
    (".worldSaveData.ItemContainerSaveData.Key", "Struct"),
    (".worldSaveData.CharacterContainerSaveData.Key", "Struct"),
];

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn open_sav_file(path: &OsString) -> io::Result<String> {
    let mut file = File::open(path)?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let compressed_data = &buffer[12..];

    let mut decompressed_data = Vec::new();
    let mut decompressor = ZlibDecoder::new(compressed_data);
    decompressor.read_to_end(&mut decompressed_data)?;

    let mut decompressed_data_cursor = io::Cursor::new(decompressed_data);
    let mut types = Types::new();
    for (path, t) in PALWORLD_TYPES.iter() {
        types.add(path.parse().unwrap(), StructType::from(t.to_string()));
    }
    match Save::read_with_types(&mut decompressed_data_cursor, &types) {
        Ok(save) => {
            let json = serde_json::to_string_pretty(&save)?;

            return Ok(json);
        }
        Err(e) => {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Failed to read save: {}", e)))
        }
    }
}

fn main() {
    let context = tauri::generate_context!();

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet])
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::FileDrop(files) => {
                    match files {
                        FileDropEvent::Dropped { paths, position }  => {
                            // get first path and open it
                            println!("Dropped files: {:?}", paths);
                            let path = paths.get(0);
                            if let Some(path) = path {
                                let os_string = path.clone().into_os_string();
                                let json = open_sav_file(&os_string);
                                match json {
                                    Ok(json) => {
                                        window.emit("sav_file", json).unwrap();
                                        window.set_title(&*format!("palworld-save-editor - {:>5?}", path.to_str().expect("Expect file path"))).unwrap();
                                    }
                                    Err(e) => {
                                        window.emit("open_err", format!("Failed to open file: {}", e)).unwrap();
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                    println!("Got files: {:?}", files);
                }
                _ => {}
            }
        })
        .build(context)
        .expect("error while running tauri application");

    app.run(|app_handle, e| match e {
        RunEvent::Ready => {
            #[cfg(debug_assertions)]
            let window = app_handle.get_window("main").unwrap();

            #[cfg(debug_assertions)]
            window.open_devtools();
        }
        _ => (),
    });
}
