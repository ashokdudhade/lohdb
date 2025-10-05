use crate::Result;
use std::collections::HashMap;

/// Trait for pluggable storage backends
pub trait StorageEngine: Send + Sync {
    /// Initialize the storage engine
    fn initialize(&mut self) -> Result<()>;
    
    /// Store a key-value pair
    fn store(&mut self, key: &str, value: &[u8]) -> Result<()>;
    
    /// Retrieve a value by key
    fn retrieve(&self, key: &str) -> Result<Option<Vec<u8>>>;
    
    /// Remove a key-value pair
    fn remove(&mut self, key: &str) -> Result<bool>;
    
    /// List all keys
    fn list_keys(&self) -> Result<Vec<String>>;
    
    /// Flush any pending writes
    fn flush(&mut self) -> Result<()>;
}

/// In-memory storage engine for testing and caching
pub struct InMemoryStorageEngine {
    data: HashMap<String, Vec<u8>>,
}

impl InMemoryStorageEngine {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl StorageEngine for InMemoryStorageEngine {
    fn initialize(&mut self) -> Result<()> {
        Ok(())
    }
    
    fn store(&mut self, key: &str, value: &[u8]) -> Result<()> {
        self.data.insert(key.to_string(), value.to_vec());
        Ok(())
    }
    
    fn retrieve(&self, key: &str) -> Result<Option<Vec<u8>>> {
        Ok(self.data.get(key).cloned())
    }
    
    fn remove(&mut self, key: &str) -> Result<bool> {
        Ok(self.data.remove(key).is_some())
    }
    
    fn list_keys(&self) -> Result<Vec<String>> {
        Ok(self.data.keys().cloned().collect())
    }
    
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

/// File-based storage engine with durability
pub struct FileStorageEngine {
    data: HashMap<String, Vec<u8>>,
    data_dir: String,
    dirty: bool,
}

impl FileStorageEngine {
    pub fn new(data_dir: String) -> Self {
        Self {
            data: HashMap::new(),
            data_dir,
            dirty: false,
        }
    }
    
    fn data_file_path(&self) -> String {
        format!("{}/data.db", self.data_dir)
    }
    
    fn load_from_disk(&mut self) -> Result<()> {
        use std::fs;
        
        let data_path = self.data_file_path();
        if !std::path::Path::new(&data_path).exists() {
            return Ok(());
        }
        
        let data = fs::read(&data_path)?;
        if !data.is_empty() {
            self.data = bincode::deserialize(&data)?;
        }
        
        Ok(())
    }
    
    fn save_to_disk(&mut self) -> Result<()> {
        if !self.dirty {
            return Ok(());
        }
        
        use std::fs;
        
        fs::create_dir_all(&self.data_dir)?;
        let data = bincode::serialize(&self.data)?;
        fs::write(self.data_file_path(), data)?;
        self.dirty = false;
        
        Ok(())
    }
}

impl StorageEngine for FileStorageEngine {
    fn initialize(&mut self) -> Result<()> {
        self.load_from_disk()
    }
    
    fn store(&mut self, key: &str, value: &[u8]) -> Result<()> {
        self.data.insert(key.to_string(), value.to_vec());
        self.dirty = true;
        Ok(())
    }
    
    fn retrieve(&self, key: &str) -> Result<Option<Vec<u8>>> {
        Ok(self.data.get(key).cloned())
    }
    
    fn remove(&mut self, key: &str) -> Result<bool> {
        let existed = self.data.remove(key).is_some();
        if existed {
            self.dirty = true;
        }
        Ok(existed)
    }
    
    fn list_keys(&self) -> Result<Vec<String>> {
        Ok(self.data.keys().cloned().collect())
    }
    
    fn flush(&mut self) -> Result<()> {
        self.save_to_disk()
    }
}