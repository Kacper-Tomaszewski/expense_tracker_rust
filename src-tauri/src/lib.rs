use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[path = "../../src/models.rs"]
pub mod models;

use models::AppState;

fn get_data_path(app: &AppHandle) -> PathBuf {
    let app_dir = app.path().app_data_dir().unwrap();
    if !app_dir.exists() {
        let _ = fs::create_dir_all(&app_dir);
    }
    app_dir.join("expense_data.json")
}

#[tauri::command]
fn save_data(app: AppHandle, state: AppState) -> Result<(), String> {
    println!("DEBUG: Próba zapisu danych..."); 

    let path = get_data_path(&app);
    let json = serde_json::to_string_pretty(&state).map_err(|e| e.to_string())?;
    
    fs::write(path, json).map_err(|e| e.to_string())?;
    
    println!("DEBUG: Zapisano pomyślnie!");
    Ok(())
}

#[tauri::command]
fn load_data(app: AppHandle) -> Result<AppState, String> {
    println!("DEBUG: Próba wczytania danych...");
    let path = get_data_path(&app);

    if !path.exists() {
        println!("DEBUG: Brak pliku, zwracam domyślne.");
        return Ok(AppState::default());
    }

    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let state = serde_json::from_str(&content).unwrap_or_default();
    
    println!("DEBUG: Wczytano dane!");
    Ok(state)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![save_data, load_data])
        .run(tauri::generate_context!())
        .expect("Błąd uruchamiania aplikacji");
}
