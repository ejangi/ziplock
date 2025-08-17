# TODO

Run one of the following prompts in Zed's agent panel and when you're ready to bump the version, use `./scripts/version/update-version.sh patch "<My feature update here>"`:


## Linux
- Taking into consideration `docs/technical.md` and `docs/architecture.md` and `docs/technical/build.md` file can you work out why the `ziplock.install` file is not being included in the release assets? Can you also remove the part of the build that publishes a `ziplock-<tag>.tar.gz` file of the source code?
- Taking into consideration `docs/technical.md` and `docs/architecture.md`, Can you please `cargo check` the linux app, look at each of the warnings and work out how to correct them? We have a shared library, so any functionality where it could be shared between platforms we should move into that location. We also need to evaluate whether each item in the warnings should still exist and is just not being used correctly. If the functionality really is redundant then please remove it.


## Android
- Taking into consideration `docs/design.md`, `docs/technical.md`, `docs/technical/android.md` and `docs/technical/CommonTemplates-FFI-Integration.md` can you implement the add/edit credentials view in the Android app so that when a user clicks on a credential in the list view, that it takes the user to the edit view. Similarly, when the user clicks on the "Add Credential" button, that it takes the user to the add view. Like the linux app, when creating a new credential, the user will have to select from a list of Credential Types first (the linux app calls these CommonTemplates).
- Taking into consideration `docs/design.md`, `docs/technical.md` and `docs/technical/android.md` can you please ensure that the Android app closes the archive correctly when the user exits the app. Is there the idea of a quick background process to finish that sort of them on Android?
