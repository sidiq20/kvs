use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    Set { key: String, value: String },
    Get { key: String },
    Remove { key: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Ok(Option<String>),
    Err(String),
}

pub struct KvStore {
    map: HashMap<String, String>,
    path: PathBuf,
}

impl KvStore {
    /// Opens the store at the given path. 
    /// If the log file exists, it replays the log to rebuild the in-memory state.
    pub fn open(path: PathBuf) -> io::Result<Self> {
        let mut map = HashMap::new();
        
        // If file exists, read it to rebuild the map
        if path.exists() {
            let file = File::open(&path)?;
            let reader = BufReader::new(file);
            
            for line in reader.lines() {
                let line = line?;
                // Parse each line as a json command
                if let Ok(cmd) = serde_json::from_str::<Command>(&line) {
                    match cmd {
                        Command::Set { key, value } => {
                            map.insert(key, value);
                    }
                    Command::Remove { key } => {
                            map.remove(&key);
                        }
                    Command::Get { .. } => {}
                    }
                }
            }
        }

        Ok(KvStore { map, path })
    }

    pub fn set(&mut self, key: String, value: String) -> io::Result<()> {
        // 1. Write to the log file first (Durability)
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
            
        let cmd = Command::Set { key: key.clone(), value: value.clone() };
        let serilized = serde_json::to_string(&cmd)?;
        writeln!(file, "{}", serilized)?;
        
        // 2. Update the in-memory map
        self.map.insert(key, value);
        Ok(())
    }

    pub fn get(&self, key: String) -> Option<String> {
        self.map.get(&key).cloned()
    }

    pub fn remove(&mut self, key: String) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .append(true)
            .open(&self.path)?;
            
        let cmd = Command::Remove { key: key.clone() };
        let serilized = serde_json::to_string(&cmd)?;
        writeln!(file, "{}", serilized)?;
        
        self.map.remove(&key);
        Ok(())
    }
}

#[derive(Clone)]
pub struct SharedKvStore(Arc<Mutex<KvStore>>);

impl SharedKvStore {
    pub fn new(store: KvStore) -> Self {
        SharedKvStore(Arc::new(Mutex::new(store)))
    }

    pub fn set(&self, key: String, value: String) -> io::Result<()> {
        self.0.lock().unwrap().set(key, value)
    }

    pub fn get(&self, key: String) -> Option<String> {
        self.0.lock().unwrap().get(key)
    }

    pub fn remove(&self, key: String) -> io::Result<()> {
        self.0.lock().unwrap().remove(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    // helper function to create a store with a temporary log file
    fn temp_store() -> (KvStore, PathBuf) {
        let mut path = temp_dir();
        // Generate a unique filename using the current timestamp
        let time  = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        path.push(format!("kvs_{}.log", time));

        (KvStore::open(path.clone()).unwrap(), path)
    }

    #[test]
    fn test_set_get() {
        let (mut store, path) = temp_store();

        store.set("key".to_string(), "value".to_string()).unwrap();

        assert_eq!(store.get("key".to_string()), Some("value".to_string()));

        // Clean up the log file
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_remove() {
        let (mut store, path) = temp_store();

        store.set("key".to_string(), "value".to_string()).unwrap();

        store.remove("key".to_string()).unwrap();

        assert_eq!(store.get("key".to_string()), None);

        //clean up the temporary file 
        let _ = std::fs::remove_file(path);
    }
}