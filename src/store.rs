use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;

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
                let parts: Vec<&str> = line.splitn(3, ',').collect();
                if parts.len() < 2 { continue; }
                
                match parts[0] {
                    "SET" => {
                        if parts.len() == 3 {
                            map.insert(parts[1].to_string(), parts[2].to_string());
                        }
                    }
                    "RM" => {
                        map.remove(parts[1]);
                    }
                    _ => {}
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
            
        writeln!(file, "SET,{},{}", key, value)?;
        
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
            
        writeln!(file, "RM,{}", key)?;
        self.map.remove(&key);
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_get() {
        let mut store = KvStore::new();
        store.set("key".to_string(), "value".to_string());
        assert_eq!(store.get("key".to_string()), Some("value".to_string()));
    }

    #[test]
    fn test_remove() {
        let mut store = KvStore::new();
        store.set("key".to_string(), "value".to_string());
        store.remove("key".to_string());
        assert_eq!(store.get("key".to_string()), None);
    }
}
