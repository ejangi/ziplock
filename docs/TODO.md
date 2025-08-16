# TODO

Run one of the following prompts in Zed's agent panel and when you're ready to bump the version, use `./scripts/version/update-version.sh patch "<My feature update here>"`:


## Linux
- Can you please `cargo check` this app, look at each of the warnings and work out how to correct them? We have a shared library, so any functionality where it could be shared between platforms we should move into that location. We also need to evaluate whether each item in the warnings should still exist and is just not being used correctly. If the functionality really is redundant then please remove it.


## Android
- Can you review `docs/technical.md` and `docs/technical/*.md` and have a think about whether we need to make any changes to file locking. Does that still work on Android? I'm conscious that sometimes the user will want to open files from Google Drive or Dropbox, etc and I'm not sure how Android handles files from cloud services and whether we still get a local storage location to manage lock files.
