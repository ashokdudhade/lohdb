use lohdb::{Database, DatabaseConfig};
use tempfile::TempDir;
use std::thread;
use std::time::Duration;

#[test]
fn test_crash_recovery() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path().to_string_lossy().to_string();
    
    let config = DatabaseConfig {
        data_dir: data_dir.clone(),
        wal_sync_interval_ms: 100,
    };
    
    // Create database and insert some data
    {
        let mut db = Database::open(config.clone()).unwrap();
        db.set("key1".to_string(), b"value1".to_vec()).unwrap();
        db.set("key2".to_string(), b"value2".to_vec()).unwrap();
        db.set("key3".to_string(), b"value3".to_vec()).unwrap();
        
        // Simulate some operations
        db.delete("key2").unwrap();
        db.set("key4".to_string(), b"value4".to_vec()).unwrap();
        
        // Force flush
        db.flush().unwrap();
    } // Database is "crashed" here when it goes out of scope
    
    // Wait a bit to simulate crash
    thread::sleep(Duration::from_millis(50));
    
    // Recover database from same directory
    {
        let db = Database::open(config).unwrap();
        
        // Verify data integrity after recovery
        assert_eq!(db.get("key1").unwrap(), Some(b"value1".to_vec()));
        assert_eq!(db.get("key2").unwrap(), None); // Was deleted
        assert_eq!(db.get("key3").unwrap(), Some(b"value3".to_vec()));
        assert_eq!(db.get("key4").unwrap(), Some(b"value4".to_vec()));
        
        let keys = db.list_keys().unwrap();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key3".to_string()));
        assert!(keys.contains(&"key4".to_string()));
        assert!(!keys.contains(&"key2".to_string()));
    }
}

#[test]
fn test_change_subscriptions() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path().to_string_lossy().to_string();
    
    let config = DatabaseConfig {
        data_dir,
        wal_sync_interval_ms: 100,
    };
    
    let mut db = Database::open(config).unwrap();
    
    let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let events_clone = events.clone();
    
    let _subscription = db.subscribe(move |event| {
        events_clone.lock().unwrap().push(format!("{:?}", event));
    }).unwrap();
    
    // Perform some operations
    db.set("test_key".to_string(), b"test_value".to_vec()).unwrap();
    db.delete("test_key").unwrap();
    
    // Give subscriber thread time to process
    thread::sleep(Duration::from_millis(100));
    
    let captured_events = events.lock().unwrap();
    assert!(captured_events.len() >= 1); // At least one event should be captured
}

#[test]
fn test_concurrent_operations() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path().to_string_lossy().to_string();
    
    let config = DatabaseConfig {
        data_dir,
        wal_sync_interval_ms: 50,
    };
    
    let db = std::sync::Arc::new(std::sync::Mutex::new(Database::open(config).unwrap()));
    
    let mut handles = vec![];
    
    // Spawn multiple threads doing operations
    for i in 0..5 {
        let db_clone = db.clone();
        let handle = thread::spawn(move || {
            for j in 0..10 {
                let key = format!("key_{}_{}", i, j);
                let value = format!("value_{}_{}", i, j).into_bytes();
                
                db_clone.lock().unwrap().set(key.clone(), value).unwrap();
                
                // Verify we can read it back
                let retrieved = db_clone.lock().unwrap().get(&key).unwrap();
                assert!(retrieved.is_some());
            }
        });
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Verify all data is present
    let keys = db.lock().unwrap().list_keys().unwrap();
    assert_eq!(keys.len(), 50); // 5 threads × 10 operations each
}