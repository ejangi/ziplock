# TODO

Run one of the following prompts in Zed's agent panel and when you're ready to bump the version, use `./scripts/version/update-version.sh patch "<My feature update here>"`:


## Linux
- Can you please `cargo check` this app, look at each of the warnings and work out how to correct them? We have a shared library, so any functionality where it could be shared between platforms we should move into that location. We also need to evaluate whether each item in the warnings should still exist and is just not being used correctly. If the functionality really is redundant then please remove it.


## Android
- Can you review `docs/technical.md`, `docs/design.md` and `docs/technical/android.md` and can we start by producing a logo spash page for the android app? It should be a simple white background with the logo (`assets/icons/ziplogo-logo.svg`)? I thought this might be a good first step towards building out the android app.
