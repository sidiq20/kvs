# Project Setup & Implementation Roadmap

This document outlines the technical design and the phased implementation plan for the Rust Key-Value Store.

## 🎯 Implementation Roadmap

### Phase 1: Foundation
- [ ] In-memory `HashMap` storage
- [ ] Simple CLI interface for basic operations (`SET`, `GET`)

### Phase 2: Persistence
- [ ] Implement Append-Only Log (AOL)
- [ ] Ensure every write operation is recorded to disk before updating memory

### Phase 3: Recovery
- [ ] Logic to replay the log on startup
- [ ] Rebuild the in-memory state from disk

### Phase 4: Networking
- [ ] Wrap the store in a TCP server
- [ ] Define a simple text-based protocol

### Phase 5: Concurrency & Optimization
- [ ] Thread safety (Mutex/RwLock)
- [ ] Log Compaction (removing redundant keys from the log)
- [ ] Indexing and Caching

---

## ⚙️ Technical Deep Dive

### 1. The Speed Layer (RAM)
We use a `HashMap<String, String>` because it provides $O(1)$ lookup.
- **SET operation**: Updates the RAM and appends to the log.
- **GET operation**: Reads directly from RAM (No disk hit = fast).

### 2. The Persistence Layer (Disk)
To ensure data isn't lost on restart, we use two strategies:
- **Append-Only Log**: Fast writes because we only append to the end of the file. No seeking or rewriting.
- **Snapshotting**: Periodically saving the entire state to a single file to keep the log replay time short.

### 3. Concurrency Handling
When multiple users hit the store, we need to manage access:
- **Simple**: `Mutex<HashMap<...>>`
- **Optimized**: `RwLock<HashMap<...>>` (allows multiple concurrent readers, but one writer)
- **Advanced**: Full async with `tokio`.

### 4. Performance Tricks
- **Compaction**: Rewriting the log file to keep only the latest value for each key.
- **Indexing**: Storing file offsets in memory to jump directly to data if needed (for disk-heavy stores).
- **Caching**: Keeping "hot" data in RAM.
