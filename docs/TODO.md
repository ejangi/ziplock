# TODO

Run one of the following prompts in Zed's agent panel and when you're ready to bump the version, use `./scripts/version/update-version.sh patch "<My feature update here>"`:

- Taking into consideration the `ziplock/docs/technical.md` file, should we look at making the IpcClient a shared component with C-bindings for use in other languages? I know that shared/src/ffi.rs exists, but I'm unsure whether it includes all the IpcClient functionality. If not, we might need to add it.
