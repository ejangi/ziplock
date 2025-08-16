# TODO

Run one of the following prompts in Zed's agent panel and when you're ready to bump the version, use `./scripts/version/update-version.sh patch "<My feature update here>"`:

- Taking into consideration `docs/*.md` docs, can you implement the clipboard timeout feature in the app so that after a TOTP code or password is copied to the clipboard the app clears the clipboard? This feature should honour the config setting clipboard_timeout which is the number of seconds to count down from.
- Taking into consideration `docs/technical/build.md`, `docs/architecture.md` and `.github/workflows/linux-build.yml`, can you please evaluate each github workflow job and step to make the whole process more efficient and faster? At the moment, I feel that it is rebuilding the project several times and taking a long time to complete.
- Can you please `cargo check` this app, look at each of the warnings and work out how to correct them? We have a shared library, so any functionality where it could be shared between platforms we should move into that location. We also need to evaluate whether each item in the warnings should still exist and is just not being used correctly. If the functionality really is redundant then please remove it.
