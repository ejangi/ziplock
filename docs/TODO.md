# TODO

Run one of the following prompts in Zed's agent panel and when you're ready to bump the version, use `./scripts/version/update-version.sh patch "<My feature update here>"`:

- Taking into consideration `docs/technical/build.md` and `.github/workflows/linux-build.yml`, can you ensure that release notes not only have the installation instructions for Ubuntu/Debian, but also build from source and also Arch/Manjaro?
- Taking into consideration `docs/technical/build.md`, can you implement a "check for updates" feature that looks for new releases? Does this kind of feature usually just check the github releases or does it need to know which app store the app was installed from?
- Taking into consideration `docs/*.md` docs, can you implement the clipboard timeout feature in the app so that after a TOTP code or password is copied to the clipboard the app clears the clipboard? This feature should honour the config setting clipboard_timeout which is the number of seconds to count down from.
