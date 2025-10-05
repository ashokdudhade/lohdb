# LohDB 🚀

A fast, durable, embeddable key-value database written in Rust with Write-Ahead Logging, crash recovery, and real-time change subscriptions.

![lohdb](./loh_db.png)


## ✨ Features

- 🔥 **High Performance**: In-memory HashMap indexing for O(1) lookups
- 💾 **Durability**: Write-Ahead Log (WAL) ensures no data loss on crashes
- 🔄 **Crash Recovery**: Automatic state restoration on startup
- 📡 **Real-time Events**: Subscribe to database changes with callbacks
- 🔌 **Pluggable Storage**: Trait-based architecture supports multiple backends
- 🛡️ **Thread Safe**: Concurrent access with proper synchronization
- 🧪 **Well Tested**: Comprehensive test suite including crash scenarios
- 📦 **Embeddable**: Zero external dependencies for core functionality

## 🚀 Quick Start

### Installation

Add LohDB to your `Cargo.toml`:

```toml
[dependencies]
lohdb = "0.1.0"
```

### Build from Source

```bash
git clone https://github.com/ashokdudhade/lohdb.git
cd lohdb
cargo build --release
```

### CLI Usage

Run the interactive CLI:

```bash
./target/release/lohdb --interactive --data-dir ./my_database
```

Example session:

```bash
lohdb> set user:1 "Alice Johnson"
✅ Set 'user:1' successfully
📡 Change: Set { key: "user:1", value: [65, 108, 105, 99, 101, 32, 74, 111, 104, 110, 115, 111, 110] }

lohdb> get user:1
📄 'user:1' = 'Alice Johnson'

lohdb> set user:2 "Bob Smith"
✅ Set 'user:2' successfully

lohdb> list
📋 Keys (2): user:1, user:2

lohdb> delete user:2
🗑️  Deleted 'user:2'

lohdb> quit
👋 Goodbye!
```

### Programmatic Usage

```rust
use lohdb::{Database, DatabaseConfig};

fn main() -> anyhow::Result<()> {
    // Configure database
    let config = DatabaseConfig {
        data_dir: "./my_database".to_string(),
        wal_sync_interval_ms: 1000,
    };
    
    // Open database (creates if doesn't exist)
    let mut db = Database::open(config)?;
    
    // Subscribe to changes
    let _subscription = db.subscribe(|event| {
        println!("Database changed: {:?}", event);
    })?;
    
    // Basic operations
    db.set("user:alice".to_string(), b"Alice".to_vec())?;
    db.set("user:bob".to_string(), b"Bob".to_vec())?;
    
    // Read data
    if let Some(value) = db.get("user:alice")? {
        println!("Found: {}", String::from_utf8_lossy(&value));
    }
    
    // List all keys
    let keys = db.list_keys()?;
    println!("All keys: {:?}", keys);
    
    // Delete data
    let existed = db.delete("user:bob")?;
    println!("Deleted bob: {}", existed);
    
    Ok(())
}
```

## 🏗️ Architecture

```
┌──────────────┐    ┌───────────────┐    ┌──────────────┐
│  CLI / API   │────▶│ Query Engine  │────▶│ Storage Layer│
└──────────────┘    └───────┬───────┘    └──────┬───────┘
                            │                   │
                    ┌───────▼────────┐  ┌───────▼────────┐
                    │ In-Memory KV   │◀─┤ WAL (Append    │
                    │ Index          │  │ Only Log)      │
                    └────────────────┘  └────────────────┘
```

### Core Components

- **Storage Engine**: Pluggable trait-based storage backends
- **Write-Ahead Log**: Durability through append-only operation logging  
- **In-Memory Index**: Fast HashMap-based key lookups
- **Event System**: Real-time change notifications via channels
- **Recovery Manager**: Automatic WAL replay on startup

## 📊 Performance

- **Writes**: ~500K ops/sec (in-memory + WAL)
- **Reads**: ~2M ops/sec (HashMap lookup)
- **Recovery**: Linear with WAL size
- **Memory**: Configurable, ~50 bytes per key overhead

## 🔌 Storage Backends

LohDB supports pluggable storage through the `StorageEngine` trait:

### Built-in Engines

- **FileStorageEngine**: Persistent disk-based storage (default)
- **InMemoryStorageEngine**: Fast in-memory storage for testing

### Custom Engines

Implement the `StorageEngine` trait for custom backends:

```rust
use lohdb::{StorageEngine, Result};

struct MyCustomEngine;

impl StorageEngine for MyCustomEngine {
    fn initialize(&mut self) -> Result<()> { /* ... */ }
    fn store(&mut self, key: &str, value: &[u8]) -> Result<()> { /* ... */ }
    fn retrieve(&self, key: &str) -> Result<Option<Vec<u8>>> { /* ... */ }
    fn remove(&mut self, key: &str) -> Result<bool> { /* ... */ }
    fn list_keys(&self) -> Result<Vec<String>> { /* ... */ }
    fn flush(&mut self) -> Result<()> { /* ... */ }
}
```

## 🔔 Change Subscriptions

Subscribe to database changes for real-time notifications:

```rust
let subscription = db.subscribe(|event| {
    match event {
        ChangeEvent::Set { key, value } => {
            println!("Key '{}' was set", key);
        }
        ChangeEvent::Delete { key } => {
            println!("Key '{}' was deleted", key);
        }
    }
})?;

// Subscription automatically cleaned up when dropped
```

## 🛡️ Durability & Recovery

### Write-Ahead Logging

Every operation is logged before execution:

1. **Write to WAL**: Operation serialized and appended to log
2. **Update Index**: In-memory state updated  
3. **Background Sync**: Periodic flush to disk

### Crash Recovery

On startup, LohDB automatically:

1. **Reads WAL**: Deserializes all logged operations
2. **Replays Operations**: Rebuilds in-memory state
3. **Resumes Normal Operation**: Database ready for use

### Data Integrity

- **Atomic Operations**: Each operation is fully logged before execution
- **Checksum Validation**: Corrupted WAL entries are detected and skipped
- **Graceful Degradation**: Partial recovery from damaged logs

## 🧪 Testing

Run the full test suite:

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test recovery

# Benchmark tests
cargo bench
```

### Test Coverage

- ✅ Basic CRUD operations
- ✅ Crash recovery scenarios  
- ✅ Concurrent access patterns
- ✅ WAL integrity and replay
- ✅ Change subscription system
- ✅ Storage engine pluggability

### Development Setup

```bash
# Clone repository
git clone https://github.com/yourusername/lohdb.git
cd lohdb

# Install dependencies
cargo build

# Run tests
cargo test

# Format code
cargo fmt

# Lint code
cargo clippy
```

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Inspired by [RocksDB](https://rocksdb.org/) and [LevelDB](https://github.com/google/leveldb)
- Built with the amazing [Rust](https://www.rust-lang.org/) ecosystem
- Special thanks to the open source community


**Built with ❤️ in Rust**