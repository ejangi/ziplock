# TODO

Run one of the following prompts in Zed's agent panel and when you're ready to bump the version, use `./scripts/version/update-version.sh patch "<My feature update here>"`:


## Linux
- Can you please `cargo check` this app, look at each of the warnings and work out how to correct them? We have a shared library, so any functionality where it could be shared between platforms we should move into that location. We also need to evaluate whether each item in the warnings should still exist and is just not being used correctly. If the functionality really is redundant then please remove it.


## Android
- Taking into consideration `docs/technical.md`, `docs/technical/android.md`, `docs/design.md` and `docs/technical/cloud-storage-implementation.md` can you develop a view that shows the list of credentials (using the FFI library to get the list of credentials). This view will appear when the user successfully unlocks an archive. I believe there is a placeholder view for that now that has a "Close Archive" button. Each credential in the list should use the Icon associated with the credential type (the linux app calls this CommonTemplates. We may need to move some of these definitions into the shared library if its not there already) on the left and then the title of the credential record. It will need a click handler, but we won't produce the add/edit credentials views yet. This list view should have a search bar at the top so the user can filter the list of credentials. We also need a way for the user to close the archive. Maybe we do a little circular "X" button in the top right corner of the view? Happy to take other suggestions to make it a good user experience.
