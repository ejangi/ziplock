# TODO

Run one of the following prompts in Zed's agent panel and when you're ready to bump the version, use `./scripts/version/update-version.sh patch "<My feature update here>"`:


## Linux
- Can you please `cargo check` this app, look at each of the warnings and work out how to correct them? We have a shared library, so any functionality where it could be shared between platforms we should move into that location. We also need to evaluate whether each item in the warnings should still exist and is just not being used correctly. If the functionality really is redundant then please remove it.


## Android
- Taking into consideration `docs/technical.md`, `docs/technical/android.md`, `docs/design.md` and `docs/technical/cloud-storage-implementation.md`, on the initial UI for the Android app, we need to replicate the linux UI by making a view where the user can select the Archive file to open and also provide a passphrase text input field so the user can supply the passphrase to unlock the archive (using the shared ffi library). Remebering, that we just pass the passphrase the library, we don't need to create any crypto code. I'm thinking as we start to build out more of the Android app, I'd like to do what we did in the linux app, where we stored a lot of the UI styling in a theme file (the linux app uses theme.rs) so we can keep fields consistent in their look and feel.
- Taking into consideration `docs/technical.md`, `docs/technical/android.md`, `docs/design.md` and `docs/technical/cloud-storage-implementation.md` can you ensure that this Android app is one of the options that the user can select to open a *.7z archive file?
