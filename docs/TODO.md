# TODO

Run one of the following prompts in Zed's agent panel and when you're ready to bump the version, use `./scripts/version/update-version.sh patch "<My feature update here>"`:

- Taking into consideration the `ziplock/README.md` and `ziplock/docs/*.md` files, should we look at making the IpcClient a shared component with C-bindings for use in other languages? I know that shared/src/ffi.rs exists, but I'm unsure whether it includes all the IpcClient functionality. If not, we might need to add it.
- Taking into consideration the `ziplock/README.md` and `ziplock/docs/*.md` files, can you hook up the one-time password field so that it stores the value, but once it has a value it displays the 6 digit code? I'm assuming there is some kind of algorithm that takes the key and can translate it into a 6 digit code that refreshes every 30 seconds? So, we might need a shared utility function(s) for that. This field will also need the visibility toggle button next to it so we can toggle between the 6 digit generated code and then underlying stored key.
