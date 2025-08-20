# TODO

Run one of the following prompts in Zed's agent panel and when you're ready to bump the version, use `./scripts/version/update-version.sh patch "<My feature update here>"`:


## Linux
- Taking into consideration `docs/technical.md` and `docs/architecture.md`, `docs/technical/android.md` and `docs/technical/android-hybrid-migration.md`, we've done a review of the code base for `shared/src/` to unify everything so the linux app also uses in the hybrid APIs to do operations in memory, but then does the extra step for non-mobile platforms (like MacOs, Linux and Windows) where the shared library does the file system operations of committing the changes to disk. I'm now at a point where I don't know what the best architecture is to keep as many of the credential and file operations in the shared library as possible, but also enable the app to work will in multiple environments. At the moment we have a linux app and an Android app. The Android app needs to do the file operations in the kotlin code as we were finding the shared library was failing to deal with the android file system. Here's the summary of what we've just finished:

Answer: **Yes, files are extracted to the regular filesystem temporarily, then loaded into memory**

Here's exactly what happens:

### 1. **Temporary Filesystem Extraction** 📁
The archive files are extracted to the **regular filesystem** in a temporary directory:

- **Location**: `/tmp/ziplock_{process_id}/` (e.g., `/tmp/ziplock_271849/`)
- **Method**: Uses the system's `7z` command to extract the archive with the provided password
- **Process**: `7z x -p{password} -y -o{temp_dir} {archive_path}`

### 2. **File Reading Into Memory** 💾
After extraction, the files are read from the temporary directory into memory:

```rust
pub fn get_extracted_files(&self) -> LinuxFileOperationResult<HashMap<String, Vec<u8>>> {
    // Recursively read all files from temp directory
    let content = fs::read(&path)?;  // Read file contents into Vec<u8>
    files.insert(relative_path.to_string_lossy().to_string(), content);
}
```

### 3. **Automatic Cleanup** 🧹
The temporary files are automatically cleaned up:

```rust
impl Drop for LinuxFileOperationsHandler {
    fn drop(&mut self) {
        if let Some(temp_dir) = &self.temp_dir {
            if temp_dir.exists() {
                if let Err(e) = fs::remove_dir_all(temp_dir) {
                    warn!("Failed to cleanup temp directory: {}", e);
                }
            }
        }
    }
}
```

## Security & Performance Implications

### **Security** 🔒
- **Temporary exposure**: Files exist on disk temporarily in `/tmp/ziplock_{pid}/`
- **Process isolation**: Each process gets its own temp directory
- **Automatic cleanup**: Files are removed when the handler is dropped
- **Filesystem permissions**: Temp files inherit system temp directory permissions

### **Performance** ⚡
- **Memory efficient**: Files are only loaded into memory after extraction
- **IO overhead**: Double I/O cost (extract to disk, then read to memory)
- **Cleanup cost**: Directory removal when done

## Alternative Approach Consideration

For better security, we could potentially:
1. **Stream directly to memory**: Extract archive contents directly to memory without filesystem intermediary
2. **Use in-memory archive libraries**: Like `sevenz-rust` (but this was causing the original async conflicts)
3. **Encrypted temporary storage**: Use encrypted temporary files

However, the current approach is **reasonable** because:
- ✅ **Temporary files are cleaned up automatically**
- ✅ **Process isolation** prevents cross-process access
- ✅ **Works reliably** without async/FFI conflicts
- ✅ **Leverages proven 7z extraction**
- ✅ **Handles complex archive structures** properly

The temporary filesystem extraction is a pragmatic solution that balances security, reliability, and performance while avoiding the async runtime conflicts that were causing the original "Database is locked" issue.


I am not happy with this approach. It doesn't conform to the original architectural objective. It seems to me we have to rewrite the `shared/` code so that it can be the single source of both asynchronous and synchronous operations. Can provide hooks for the hybrid scenario in Mobile (where the shared library does operations on the archive file structure in memory using yml files and hands off archive open/save via the mobile code - kotlin on android) and also the direct scenario for desktop where it does the operations in memory, but then can work with the archive files on disk as well.

- Taking into consideration `docs/technical.md` and `docs/architecture.md`, `docs/technical/android.md` and `docs/technical/android-hybrid-migration.md`, we've done a review of the code base for `shared/src/` to unify everything so the linux app uses the shared code for the most part, but then does the extra step for non-mobile platforms (like MacOs, Linux and Windows) where the shared library does the file system operations of committing the changes to disk.  Here's the summary of what we've just finished:

This solution eliminates the hanging issue while maintaining the unified hybrid architecture across all platforms.

I can see the main view is trying to use the hybrid client to load credentials, which is causing the "Database is locked" issue. Let me create a simple credential storage mechanism that works directly with the extracted files to avoid the FFI altogether.

New file `apps/linux/src/services/credential_store.rs` created.

Updated the services module `apps/linux/src/services/mod.rs`.

Can you please continue this work?

- Taking into consideration `docs/technical.md` and `docs/architecture.md`, can you ensure the app is create logs and has the necessary log rotate setup when it's deployed. Not sure if this needs to be in the build or part of the codebase.


## Android
- Taking into consideration `docs/design.md`, `docs/technical.md` and `docs/technical/android.md` can you please ensure that the Android app closes the archive correctly when the user exits the app. Is there the idea of a quick background process to finish that sort of them on Android?


## Windows
- Taking into consideration `docs/design.md`, `docs/architecture.md`, `docs/technical.md` can we look at the viability of using Iced as the GUI for Windows instead of Tauri? If that is possible can you update the documentation and the README.md to reflect the change?
