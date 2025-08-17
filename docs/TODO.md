# TODO

Run one of the following prompts in Zed's agent panel and when you're ready to bump the version, use `./scripts/version/update-version.sh patch "<My feature update here>"`:


## Linux
- Can you please `cargo check` this app, look at each of the warnings and work out how to correct them? We have a shared library, so any functionality where it could be shared between platforms we should move into that location. We also need to evaluate whether each item in the warnings should still exist and is just not being used correctly. If the functionality really is redundant then please remove it.
- Taking into consideration `docs/technical.md`, can you ensure that the linux app is one of the options for *.7z files in the "Open with..." option when you right click on a 7z file?


## Android
- Taking into consideration `docs/technical.md`, `docs/technical/android.md`, `docs/design.md` and `docs/technical/cloud-storage-implementation.md` can you ensure that this Android app is one of the options that the user can select to open a *.7z archive file?
