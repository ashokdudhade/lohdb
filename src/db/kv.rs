use crate::db::{
    StorageEngine, FileStorageEngine, WriteAheadLog, Operation,
    EventBus, ChangeEvent, SubscriptionHandle, Subscriber
};
use crate::Result;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::thread;

pub struct DatabaseConfig {
    pub data_dir: String,
    pub wal_sync_interval_ms: u64,
}

pub struct Database {
    storage: Arc<Mutex<Box<dyn StorageEngine>>>,
    wal: Arc<Mutex<WriteAheadLog>>,
    event_bus: Arc<Mutex<EventBus>>,
    _sync_handle: thread::JoinHandle<()>,
}

impl Database {
    pub fn open(config: DatabaseConfig) -> Result<Self> {
        let mut storage: Box<dyn StorageEngine> = Box::new(FileStorageEngine::new(config.data_dir.clone()));
        storage.initialize()?;
        
        let wal_path = format!("{}/wal.log", config.data_dir);
        let mut wal = WriteAheadLog::new(&wal_path)?;
        
        // Replay WAL to restore state
        let storage_for_replay = Arc::new(Mutex::new(storage));
        {
            let storage_clone = storage_for_replay.clone();
            wal.replay(|operation| {
                let mut storage = storage_clone.lock().unwrap();
                match operation {
                    Operation::Set { key, value } => {
                        storage.store(&key, &value)?;
                    }
                    Operation::Delete { key } => {
                        storage.remove(&key)?;
                    }
                }
                Ok(())
            })?;
        }
        
        let wal = Arc::new(Mutex::new(wal));
        let event_bus = Arc::new(Mutex::new(EventBus::new()));
        
        // Start background sync thread
        let storage_for_sync = storage_for_replay.clone();
        let sync_handle = thread::spawn(move || {
            let interval = Duration::from_millis(config.wal_sync_interval_ms);
            let mut last_sync = Instant::now();
            
            loop {
                thread::sleep(Duration::from_millis(100));
                
                if last_sync.elapsed() >= interval {
                    if let Ok(mut storage) = storage_for_sync.lock() {
                        let _ = storage.flush();
                    }
                    last_sync = Instant::now();
                }
            }
        });
        
        Ok(Self {
            storage: storage_for_replay,
            wal,
            event_bus,
            _sync_handle: sync_handle,
        })
    }
    
    pub fn set(&mut self, key: String, value: Vec<u8>) -> Result<()> {
        let operation = Operation::Set {
            key: key.clone(),
            value: value.clone(),
        };
        
        // Write to WAL first
        self.wal.lock().unwrap().append(&operation)?;
        
        // Then update storage
        self.storage.lock().unwrap().store(&key, &value)?;
        
        // Publish change event
        let event = ChangeEvent::Set { key, value };
        self.event_bus.lock().unwrap().publish(event)?;
        
        Ok(())
    }
    
    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        self.storage.lock().unwrap().retrieve(key)
    }
    
    pub fn delete(&mut self, key: &str) -> Result<bool> {
        let operation = Operation::Delete {
            key: key.to_string(),
        };
        
        // Write to WAL first
        self.wal.lock().unwrap().append(&operation)?;
        
        // Then update storage
        let existed = self.storage.lock().unwrap().remove(key)?;
        
        if existed {
            // Publish change event
            let event = ChangeEvent::Delete {
                key: key.to_string(),
            };
            self.event_bus.lock().unwrap().publish(event)?;
        }
        
        Ok(existed)
    }
    
    pub fn list_keys(&self) -> Result<Vec<String>> {
        self.storage.lock().unwrap().list_keys()
    }
    
    pub fn subscribe<F>(&mut self, callback: F) -> Result<SubscriptionHandle>
    where
        F: Fn(ChangeEvent) + Send + Sync + 'static,
    {
        self.event_bus.lock().unwrap().subscribe(callback)
    }
    
    pub fn flush(&mut self) -> Result<()> {
        self.storage.lock().unwrap().flush()
    }
}

// Implement Send and Sync manually since we know our implementation is thread-safe
unsafe impl Send for Database {}
unsafe impl Sync for Database {}