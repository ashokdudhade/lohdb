use crate::{Database, Result};
use std::io::{self, Write};

pub fn run_cli(mut db: Database) -> Result<()> {
    println!("LohDB Interactive CLI");
    println!("Commands: set <key> <value>, get <key>, delete <key>, list, quit");
    
    // Subscribe to changes for demo
    let _subscription = db.subscribe(|event| {
        println!("📡 Change: {:?}", event);
    })?;
    
    loop {
        print!("lohdb> ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        
        match parts[0].to_lowercase().as_str() {
            "set" if parts.len() == 3 => {
                let key = parts[1].to_string();
                let value = parts[2].as_bytes().to_vec();
                match db.set(key.clone(), value) {
                    Ok(_) => println!("✅ Set '{}' successfully", key),
                    Err(e) => println!("❌ Error: {}", e),
                }
            }
            "get" if parts.len() == 2 => {
                let key = parts[1];
                match db.get(key) {
                    Ok(Some(value)) => {
                        match String::from_utf8(value) {
                            Ok(s) => println!("📄 '{}' = '{}'", key, s),
                            Err(_) => println!("📄 '{}' = <binary data>", key),
                        }
                    }
                    Ok(None) => println!("🔍 Key '{}' not found", key),
                    Err(e) => println!("❌ Error: {}", e),
                }
            }
            "delete" if parts.len() == 2 => {
                let key = parts[1];
                match db.delete(key) {
                    Ok(existed) => {
                        if existed {
                            println!("🗑️  Deleted '{}'", key);
                        } else {
                            println!("🔍 Key '{}' not found", key);
                        }
                    }
                    Err(e) => println!("❌ Error: {}", e),
                }
            }
            "list" => {
                match db.list_keys() {
                    Ok(keys) => {
                        if keys.is_empty() {
                            println!("📭 Database is empty");
                        } else {
                            println!("📋 Keys ({}): {}", keys.len(), keys.join(", "));
                        }
                    }
                    Err(e) => println!("❌ Error: {}", e),
                }
            }
            "quit" | "exit" => {
                println!("👋 Goodbye!");
                break;
            }
            _ => {
                println!("❓ Unknown command. Available: set, get, delete, list, quit");
            }
        }
    }
    
    Ok(())
}