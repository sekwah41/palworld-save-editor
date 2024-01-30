// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::ffi::OsString;
use std::{env, fs, thread};
use tauri::{FileDropEvent, Manager, RunEvent};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::Command;
use flate2::read::ZlibDecoder;
use uesave::{Save, StructType, Types};
use byteorder::{LittleEndian, ReadBytesExt};

const PALWORLD_TYPES: [(&str, &str); 6] = [
    (".worldSaveData.CharacterSaveParameterMap.Key", "Struct"),
    (".worldSaveData.FoliageGridSaveDataMap.Key", "Struct"),
    (".worldSaveData.FoliageGridSaveDataMap.ModelMap.InstanceDataMap.Key", "Struct"),
    (".worldSaveData.MapObjectSpawnerInStageSaveData.Key", "Struct"),
    (".worldSaveData.ItemContainerSaveData.Key", "Struct"),
    (".worldSaveData.CharacterContainerSaveData.Key", "Struct"),
];

#[derive(serde::Serialize, serde::Deserialize)]
struct WindowConfig {
    x: i32,
    y: i32,
}


#[cfg(debug_assertions)]
fn read_config() -> std::io::Result<WindowConfig> {
    let path = Path::new("../window_config.json");
    if path.exists() {
        let data = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&data)?)
    } else {
        Ok(WindowConfig { x: 0, y: 0 })
    }
}

#[cfg(debug_assertions)]
fn write_config(config: &WindowConfig) -> std::io::Result<()> {
    let data = serde_json::to_string(config)?;
    fs::write("../window_config.json", data)?;
    Ok(())
}

#[tauri::command]
fn open_saves_folder() {
    if let Ok(appdata) = env::var("LOCALAPPDATA") {
        let directory = format!("{}\\Pal\\Saved\\SaveGames", appdata);

        println!("Trying to open {}", directory);

        Command::new("explorer")
            .arg(directory)
            .spawn()
            .expect("Failed to open directory");
    } else {
        println!("APPDATA environment variable is not set.");
    }
}

// The current logic of saving and loading is copied from the palworld-host-save-fix
#[tauri::command]
fn save_file(json: &str, save_type: u8, path: &str) -> String {
    println!("Saving file: {}", path);
    if path == "" {
        return "No file path provided!".to_string();
    }
    let save = serde_json::from_str::<Save>(json).unwrap();

    let mut reconstructed: Vec<u8> = vec![];
    if let Err(err) = save.write(&mut reconstructed) {
        return format!("Failed to write save: {}", err);
    }

    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    if let Err(err) = encoder.write_all(&reconstructed) {
        return format!("Failed to compress save: {}", err);
    }
    let compressed_data = encoder.finish();
    if let Err(err) = compressed_data {
        return format!("Failed to finish compression: {}", err);
    }
    let compressed_data = compressed_data.unwrap();

    let file = File::create(format!("{}test",path));
    if let Err(err) = file {
        return format!("Failed to create file: {}", err);
    }

    let mut file = file.unwrap();
    if let Err(err) = file.write_all(&*compressed_data) {
        return format!("Failed to write file: {}", err);
    }

    return "Saved file successfully!".to_string();
}

fn open_sav_file(path: &OsString) -> io::Result<(String, u8)> {
    let mut file = File::open(path)?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let uncompressed_len = (&buffer[..4]).read_u32::<LittleEndian>()?;
    let compressed_len = (&buffer[4..8]).read_u32::<LittleEndian>()?;
    let magic_bytes = &buffer[8..11];
    let save_type = &buffer[11];

    let compressed_data = &buffer[12..];

    println!("Uncompressed len: {}", uncompressed_len);
    println!("Compressed len: {}", compressed_len);
    println!("Magic bytes: {}", String::from_utf8_lossy(magic_bytes));
    println!("Save type: 0x{:02X}", save_type);

    if magic_bytes != b"PlZ" {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid magic bytes"));
    }

    if ![0x30, 0x31, 0x32].contains(save_type) {
        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid save type 0x{:02X}", save_type)));
    }

    if save_type == &0x31u8 && compressed_len != (buffer.len() as u32 - 12) {
        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid uncompressed len {} != {}", compressed_len, buffer.len())));
    }

    println!("Decompressing...");
    let mut decompressed_data = Vec::new();
    let mut decompressor = ZlibDecoder::new(compressed_data);
    decompressor.read_to_end(&mut decompressed_data)?;

    println!("Decompressed len: {}", decompressed_data.len());

    if save_type == &0x32u8 {
        if compressed_len != (decompressed_data.len() as u32) {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid uncompressed len {} != {}", compressed_len, buffer.len())));
        }
        let mut double_decompressed_data = Vec::new();
        let mut double_decompressor = ZlibDecoder::new(decompressed_data.as_slice());
        double_decompressor.read_to_end(&mut double_decompressed_data)?;
        decompressed_data = double_decompressed_data;
    }

    let mut decompressed_data_cursor = io::Cursor::new(decompressed_data);
    let mut types = Types::new();
    for (path, t) in PALWORLD_TYPES.iter() {
        types.add(path.parse().unwrap(), StructType::from(t.to_string()));
    }

    return match Save::read_with_types(&mut decompressed_data_cursor, &types) {
        Ok(save) => {
            let json = serde_json::to_string(&save)?;

            Ok((json, save_type.clone()))
        }
        Err(e) => {
            Err(io::Error::new(io::ErrorKind::InvalidData, format!("Failed to read save: {}", e)))
        }
    }
}

fn main() {
    let context = tauri::generate_context!();

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![save_file, open_saves_folder])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_window("main").unwrap();

                // Load and set window position and size in debug mode
                if let Ok(config) = read_config() {
                    window.set_position(tauri::PhysicalPosition::new(config.x, config.y))?;
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::FileDrop(files) => {
                    match files {
                        FileDropEvent::Dropped { paths, .. } => {
                            // get first path and open it
                            println!("Dropped files: {:?}", paths);
                            let path = paths.get(0);
                            if let Some(path) = path {
                                let os_string = path.clone().into_os_string();
                                window.emit("sav_file", ("{\"status\": \"Loading file...\"", 0)).unwrap();
                                window.emit("sav_path", "").unwrap();
                                let window = window.clone();
                                thread::spawn(move || {
                                    let json = open_sav_file(&os_string);
                                    match json {
                                        Ok(json) => {
                                            window.emit("sav_file", json).unwrap();
                                            window.emit("sav_path", os_string.to_str().expect("Expect file path")).unwrap();
                                            window.set_title(&*format!("palworld-save-editor - {:>5?}", os_string.to_str().expect("Expect file path"))).unwrap();
                                        }
                                        Err(e) => {
                                            window.emit("open_err", format!("Failed to open file: {}", e)).unwrap();
                                        }
                                    }
                                });
                            }
                        }
                        _ => {}
                    }
                    println!("Got files: {:?}", files);
                }
                #[cfg(debug_assertions)]
                tauri::WindowEvent::Moved(position) => {
                    let config = WindowConfig {
                        x: position.x,
                        y: position.y,
                    };
                    let _ = write_config(&config);
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
