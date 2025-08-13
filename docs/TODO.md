# TODO

- Taking into consideration the `ziplock/README.md` and `ziplock/docs/*.md` files, should we look at making the IpcClient a shared component with C-bindings for use in other languages?
- Taking into consideration the `ziplock/README.md` and `ziplock/docs/*.md` files, can you hook up the Delete button on a credential so the record is deleted, the user is taken back to the list of (refreshed) credentials along with a toast to say that the item was successfully deleted?
- Taking into consideration the `ziplock/README.md` and `ziplock/docs/*.md` files, can you ensure that when the backend first connects to a repository that it runs validations against the archive and it's contents to ensure that the archive is valid and that the contents are not corrupted? I believe there are already methods for validation that are currently reporting is unused when we build the backend.
- Taking into consideration the `ziplock/README.md` and `ziplock/docs/*.md` files, can you hook up the one-time password field so that it stores the value, but once it has a value it displays the 6 digit code? I'm assuming there is some kind of algorithm that takes the key and can translate it into a 6 digit code that refreshes every 30 seconds? So, we might need a shared utility function(s) for that. This field will also need the visibility toggle button next to it so we can toggle between the 6 digit generated code and then underlying stored key.
- Taking into consideration the `ziplock/README.md` and `ziplock/docs/*.md` files, can you make sure that the IPC frontend is using a consistent struct for storing the client session information? Currently, I'm seeing the following in the build that suggest we either have dead code that needs cleaning up or we are not using the struct consistently:
```
warning: fields `session_id` and `client_info` are never read
  --> backend/src/ipc/mod.rs:41:5
   |
40 | struct ClientSession {
   |        ------------- fields in this struct
41 |     session_id: String,
   |     ^^^^^^^^^^
...
44 |     client_info: Option<String>,
   |     ^^^^^^^^^^^
   |
   = note: `ClientSession` has a derived impl for the trait `Debug`, but this is intentionally ignored during dead code analysis
   = note: `#[warn(dead_code)]` on by default
```
- Taking into consideration the `ziplock/README.md` and `ziplock/docs/*.md` files,  can you ensure we are always using a consistent struct for storing the file lock. Currently in the build logs, I'm seeing the following that suggests we either have dead code that needs cleaning up or we are not using the struct consistently:
```
warning: field `file_lock` is never read
  --> backend/src/storage/mod.rs:50:5
   |
46 | struct OpenArchive {
   |        ----------- field in this struct
...
50 |     file_lock: FileLock,
   |     ^^^^^^^^^
   |
   = note: `OpenArchive` has a derived impl for the trait `Debug`, but this is intentionally ignored during dead code analysis
```
- Taking into consideration `docs/technical.md` can you please consolidate `docs/build.md` and `docs/technical/build-linux.md` into a single file called `docs/technical/build.md`? Can you please ensure that the all references to these files are cleaned up to point to the new file?
- Taking into consideration `docs/technical.md` and `docs/technical/build.md`, can you please cleanup the `scipts` folder so that scripts are put in the correct sub-folders, document these subfolders in the `docs/technical.md` file and ensure all references to these script files in the project are updated to reference the new location.
