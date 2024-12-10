use std::fs;
use std::path::PathBuf;
use uuid::Uuid;
use directories::BaseDirs;
use clap::{Parser, Subcommand};
use std::process::Command;
use chrono::Local;
use serde_json::{Value, json};
use sha2::{Sha256, Digest};
use rand::{thread_rng, Rng};
use hex;

#[derive(Parser)]
#[command(name = "kurzorisdead")]
#[command(about = "Narzędzie wiersza poleceń do zarządzania plikami konfiguracyjnymi Cursor", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Wyświetl informacje o ID urządzenia
    Ids,
    /// Wygeneruj losowe ID urządzenia
    RandomIds,
    /// Usuń plik ID urządzenia
    Delete,
    /// Zakończ wszystkie procesy Cursor
    Kill,
}

struct CursorIds {
    machine_id: String,
    mac_machine_id: String,
    telemetry_machine_id: String,
    dev_device_id: String,
}

fn generate_random_ids() -> CursorIds {
    // Generate UUID for machine_id and dev_device_id
    let uuid = Uuid::new_v4().to_string();
    
    // Generate random MAC-like string for mac_machine_id
    let mut rng = thread_rng();
    let mac: Vec<u8> = (0..6).map(|_| rng.gen()).collect();
    let mac_str = mac.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<String>>()
        .join(":");
    
    // Generate SHA256 hashes for machine IDs
    let mut hasher = Sha256::new();
    hasher.update(&mac_str);
    let mac_machine_id = hex::encode(hasher.finalize());
    
    let mut hasher = Sha256::new();
    hasher.update(uuid.as_bytes());
    let telemetry_machine_id = hex::encode(hasher.finalize());
    
    CursorIds {
        machine_id: uuid.clone(),
        mac_machine_id,
        telemetry_machine_id,
        dev_device_id: uuid,
    }
}

fn get_storage_path() -> Option<PathBuf> {
    if let Some(base_dirs) = BaseDirs::new() {
        let app_data_dir = if cfg!(windows) {
            base_dirs.data_local_dir()
        } else {
            base_dirs.config_dir()
        };

        let possible_paths = vec![
            // Windows: %LOCALAPPDATA%\Programs\cursor\User\globalStorage\storage.json
            // Linux/Mac: ~/.config/Cursor/User/globalStorage/storage.json
            {
                let mut path = PathBuf::from(app_data_dir);
                if cfg!(windows) {
                    path.push("Programs");
                    path.push("cursor");
                } else {
                    path.push("Cursor");
                }
                path.push("User");
                path.push("globalStorage");
                path.push("storage.json");
                path
            },
            // Alternative paths for different installations
            {
                let mut path = PathBuf::from(app_data_dir);
                if cfg!(windows) {
                    path.push("cursor");
                } else {
                    path.push(".cursor");
                }
                path.push("User");
                path.push("globalStorage");
                path.push("storage.json");
                path
            },
        ];

        for path in possible_paths {
            if path.exists() {
                return Some(path);
            }
        }

        // Return default path if no existing file found
        let mut default_path = PathBuf::from(app_data_dir);
        if cfg!(windows) {
            default_path.push("Programs");
            default_path.push("cursor");
        } else {
            default_path.push("Cursor");
        }
        default_path.push("User");
        default_path.push("globalStorage");
        default_path.push("storage.json");
        Some(default_path)
    } else {
        None
    }
}

fn get_machine_id_path() -> Option<PathBuf> {
    if let Some(base_dirs) = BaseDirs::new() {
        let app_data_dir = if cfg!(windows) {
            base_dirs.data_local_dir()
        } else {
            base_dirs.config_dir()
        };

        let possible_paths = vec![
            // Windows: %LOCALAPPDATA%\Programs\cursor\machineid
            // Linux/Mac: ~/.config/Cursor/machineid
            {
                let mut path = PathBuf::from(app_data_dir);
                if cfg!(windows) {
                    path.push("Programs");
                    path.push("cursor");
                } else {
                    path.push("Cursor");
                }
                path.push("machineid");
                path
            },
            // Alternative paths
            {
                let mut path = PathBuf::from(app_data_dir);
                if cfg!(windows) {
                    path.push("cursor");
                } else {
                    path.push(".cursor");
                }
                path.push("machineid");
                path
            },
        ];

        for path in possible_paths {
            if path.exists() {
                return Some(path);
            }
        }

        // Return default path if no existing file found
        let mut default_path = PathBuf::from(app_data_dir);
        if cfg!(windows) {
            default_path.push("Programs");
            default_path.push("cursor");
        } else {
            default_path.push("Cursor");
        }
        default_path.push("machineid");
        Some(default_path)
    } else {
        None
    }
}

fn create_backup(file_path: &PathBuf) -> Result<(), std::io::Error> {
    if file_path.exists() {
        let content = fs::read_to_string(file_path)?;
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let mut backup_path = file_path.clone();
        backup_path.set_file_name(format!("{}_backup_{}", 
            file_path.file_name().unwrap().to_string_lossy(),
            timestamp));
        fs::write(&backup_path, content)?;
        println!("Utworzono kopię zapasową: {:?}", backup_path);
    }
    Ok(())
}

fn show_machine_id() {
    if let Some(machine_id_path) = get_machine_id_path() {
        println!("Ścieżka pliku machineid: {:?}", machine_id_path);
        if let Ok(content) = fs::read_to_string(&machine_id_path) {
            println!("Aktualne machineid to: {}", content.trim());
        }
    }

    if let Some(storage_path) = get_storage_path() {
        println!("\nŚcieżka pliku storage.json: {:?}", storage_path);
        if let Ok(content) = fs::read_to_string(&storage_path) {
            if let Ok(json) = serde_json::from_str::<Value>(&content) {
                println!("telemetry.macMachineId: {}", 
                    json.get("telemetry.macMachineId").and_then(Value::as_str).unwrap_or("nie znaleziono"));
                println!("telemetry.machineId: {}", 
                    json.get("telemetry.machineId").and_then(Value::as_str).unwrap_or("nie znaleziono"));
                println!("telemetry.devDeviceId: {}", 
                    json.get("telemetry.devDeviceId").and_then(Value::as_str).unwrap_or("nie znaleziono"));
            }
        }
    }
}

fn generate_random_id() {
    let ids = generate_random_ids();
    
    // Update machineid file
    if let Some(machine_id_path) = get_machine_id_path() {
        if machine_id_path.exists() {
            if let Err(e) = create_backup(&machine_id_path) {
                eprintln!("Błąd tworzenia kopii zapasowej machineid: {}", e);
                return;
            }
        }

        if let Some(parent) = machine_id_path.parent() {
            fs::create_dir_all(parent).unwrap_or_else(|e| {
                eprintln!("Błąd tworzenia katalogu: {}", e);
            });
        }
        
        if let Err(e) = fs::write(&machine_id_path, &ids.machine_id) {
            eprintln!("Błąd zapisu pliku machineid: {}", e);
            return;
        }
        println!("Zaktualizowano machineid: {}", ids.machine_id);
    }

    // Update storage.json
    if let Some(storage_path) = get_storage_path() {
        if storage_path.exists() {
            if let Err(e) = create_backup(&storage_path) {
                eprintln!("Błąd tworzenia kopii zapasowej storage.json: {}", e);
                return;
            }
        }

        let mut storage_content = json!({
            "telemetry.macMachineId": ids.mac_machine_id,
            "telemetry.machineId": ids.telemetry_machine_id,
            "telemetry.devDeviceId": ids.dev_device_id
        });

        // If file exists, merge with existing content
        if storage_path.exists() {
            if let Ok(content) = fs::read_to_string(&storage_path) {
                if let Ok(mut existing_json) = serde_json::from_str::<Value>(&content) {
                    if let Some(obj) = existing_json.as_object_mut() {
                        obj.insert("telemetry.macMachineId".to_string(), json!(ids.mac_machine_id));
                        obj.insert("telemetry.machineId".to_string(), json!(ids.telemetry_machine_id));
                        obj.insert("telemetry.devDeviceId".to_string(), json!(ids.dev_device_id));
                        storage_content = existing_json;
                    }
                }
            }
        }

        if let Some(parent) = storage_path.parent() {
            fs::create_dir_all(parent).unwrap_or_else(|e| {
                eprintln!("Błąd tworzenia katalogu: {}", e);
            });
        }

        if let Err(e) = fs::write(&storage_path, 
            serde_json::to_string_pretty(&storage_content).unwrap_or_default()) {
            eprintln!("Błąd zapisu pliku storage.json: {}", e);
            return;
        }
        println!("\nZaktualizowano storage.json:");
        println!("telemetry.macMachineId: {}", ids.mac_machine_id);
        println!("telemetry.machineId: {}", ids.telemetry_machine_id);
        println!("telemetry.devDeviceId: {}", ids.dev_device_id);
    }
}

fn delete_machine_id() {
    if let Some(file_path) = get_machine_id_path() {
        println!("Czy na pewno chcesz usunąć plik ID urządzenia? [t/N]");
        
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_ok() {
            if input.trim().to_lowercase() == "t" {
                if let Err(e) = create_backup(&file_path) {
                    eprintln!("Błąd tworzenia kopii zapasowej: {}", e);
                    return;
                }
                
                match fs::remove_file(&file_path) {
                    Ok(_) => println!("Plik ID urządzenia został usunięty"),
                    Err(e) => eprintln!("Błąd usuwania pliku: {}", e)
                }
            } else {
                println!("Anulowano usuwanie");
            }
        } else {
            eprintln!("Błąd odczytu danych wejściowych");
        }
    }
}

fn kill_cursor_processes() {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("taskkill")
            .args(["/F", "/IM", "cursor.exe"])
            .output();
        
        match output {
            Ok(output) => {
                if output.status.success() {
                    println!("Pomyślnie zakończono wszystkie procesy Cursor");
                } else {
                    println!("Nie znaleziono uruchomionych procesów Cursor");
                }
            }
            Err(e) => eprintln!("Błąd wykonania polecenia: {}", e),
        }
    }

    #[cfg(target_os = "macos")]
    {
        let output = Command::new("pkill")
            .arg("cursor")
            .output();
        
        match output {
            Ok(output) => {
                if output.status.success() {
                    println!("Pomyślnie zakończono wszystkie procesy Cursor");
                } else {
                    println!("Nie znaleziono uruchomionych procesów Cursor");
                }
            }
            Err(e) => eprintln!("Błąd wykonania polecenia: {}", e),
        }
    }

    #[cfg(target_os = "linux")]
    {
        let output = Command::new("pkill")
            .arg("cursor")
            .output();
        
        match output {
            Ok(output) => {
                if output.status.success() {
                    println!("Pomyślnie zakończono wszystkie procesy Cursor");
                } else {
                    println!("Nie znaleziono uruchomionych procesów Cursor");
                }
            }
            Err(e) => eprintln!("Błąd wykonania polecenia: {}", e),
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Ids => {
            show_machine_id();
        }
        Commands::RandomIds => {
            generate_random_id();
        }
        Commands::Delete => {
            delete_machine_id();
        }
        Commands::Kill => {
            kill_cursor_processes();
        }
    }
}
