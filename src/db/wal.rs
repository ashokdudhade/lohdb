use crate::Result;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Write, Seek, SeekFrom};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    Set { key: String, value: Vec<u8> },
    Delete { key: String },
}

pub struct WriteAheadLog {
    file: File,
    path: String,
}

impl WriteAheadLog {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(&path)?;
            
        Ok(Self {
            file,
            path: path_str,
        })
    }
    
    pub fn append(&mut self, operation: &Operation) -> Result<()> {
        let serialized = bincode::serialize(operation)?;
        let len = serialized.len() as u32;
        
        // Write length prefix followed by the operation
        self.file.write_all(&len.to_le_bytes())?;
        self.file.write_all(&serialized)?;
        self.file.flush()?;
        
        Ok(())
    }
    
    pub fn replay<F>(&mut self, mut callback: F) -> Result<()>
    where
        F: FnMut(Operation) -> Result<()>,
    {
        use std::io::Read;
        
        // Seek to beginning of file
        self.file.seek(SeekFrom::Start(0))?;
        
        let mut len_buf = [0u8; 4];
        
        loop {
            // Try to read the length prefix
            match self.file.read_exact(&mut len_buf) {
                Ok(()) => {
                    let len = u32::from_le_bytes(len_buf) as usize;
                    let mut operation_buf = vec![0u8; len];
                    
                    self.file.read_exact(&mut operation_buf)?;
                    
                    match bincode::deserialize::<Operation>(&operation_buf) {
                        Ok(operation) => callback(operation)?,
                        Err(e) => {
                            eprintln!("Warning: Failed to deserialize WAL entry: {}", e);
                            break;
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    // End of file reached
                    break;
                }
                Err(e) => return Err(e.into()),
            }
        }
        
        // Seek back to end for future appends
        self.file.seek(SeekFrom::End(0))?;
        
        Ok(())
    }
    
    pub fn truncate(&mut self) -> Result<()> {
        use std::fs;
        
        // Close current file and recreate it empty
        fs::remove_file(&self.path)?;
        
        self.file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(&self.path)?;
            
        Ok(())
    }
}