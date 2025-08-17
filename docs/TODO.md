# TODO

Run one of the following prompts in Zed's agent panel and when you're ready to bump the version, use `./scripts/version/update-version.sh patch "<My feature update here>"`:


## Linux
- Taking into consideration `docs/design.md`, `docs/technical.md`, `docs/02-specification.md`, can you please move the CommonTemplates into the shared library so we can keep a consistent set of Credential Types across platforms?
- Taking into consideration `docs/technical.md` and `docs/architecture.md`, Can you please `cargo check` the linux app, look at each of the warnings and work out how to correct them? We have a shared library, so any functionality where it could be shared between platforms we should move into that location. We also need to evaluate whether each item in the warnings should still exist and is just not being used correctly. If the functionality really is redundant then please remove it.


## Android
- Taking into consideration `docs/design.md`, `docs/technical.md` and `docs/technical/android.md` when I get to the list of credentials and click the "refresh" button, it shows the mock credentials. Instead, if there are no credentials, it should show the view that already exists for when there are no credentials present (the one where it has the button to "Add a Credential").
- Taking into consideration `docs/design.md`, `docs/technical.md` and `docs/technical/android.md` can you please ensure that the Android app closes the archive correctly when the user exits the app. Is there the idea of a quick background process to finish that sort of them on Android?
- Taking into consideration `docs/design.md`, `docs/technical.md` and `docs/technical/android.md` can you implement the add/edit credentials view in the Android app so that when a user clicks on a credential in the list view, that it takes the user to the edit view. Similarly, when the user clicks on the "Add Credential" button, that it takes the user to the add view. Like the linux app, when creating a new credential, the user will have to select from a list of Credential Types first (the linux app calls these CommonTemplates).
