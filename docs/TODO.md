# TODO

Run one of the following prompts in Zed's agent panel and when you're ready to bump the version, use `./scripts/version/update-version.sh patch "<My feature update here>"`:


## Linux
- Can you please `cargo check` this app, look at each of the warnings and work out how to correct them? We have a shared library, so any functionality where it could be shared between platforms we should move into that location. We also need to evaluate whether each item in the warnings should still exist and is just not being used correctly. If the functionality really is redundant then please remove it.


## Android
- Taking into consideration `docs/technical.md`, `docs/technical/android.md`, `docs/design.md` and `docs/technical/cloud-storage-implementation.md` can you please ensure that once a user has selected an archive file to open that the app keeps a persistent memory of the file path so that each subsequent time the app is opened the user only needs to enter their passphrase and click "Open".
- Taking into consideration `docs/technical.md`, `docs/technical/android.md`, `docs/design.md` and `docs/technical/cloud-storage-implementation.md` can you ensure that this Android app is one of the options that the user can select to open a *.7z archive file?
