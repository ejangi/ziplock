# TODO

Run one of the following prompts in Zed's agent panel and when you're ready to bump the version, use `./scripts/version/update-version.sh patch "<My feature update here>"`:

- Taking into consideration `docs/technical.md` and `docs/technical/build.md` and remembering to only output summary information (not create new *.md files), can you please run the full suite of rust and Android tests to check for issues before I push to github?


## Linux
- Taking into consideration `docs/technical.md` and `docs/architecture.md`, can you ensure the app is creating logs and has the necessary log rotate setup when it's deployed. Not sure if this needs to be in the build or part of the codebase.


## Android
- Taking into consideration `docs/design.md`, `docs/technical.md` and `docs/technical/*.md` and remembering to only output summary information (not create new *.md files), can you please...



## Windows
- Taking into consideration `docs/design.md`, `docs/architecture.md`, `docs/technical.md` can we look at the viability of using Iced as the GUI for Windows instead of Tauri? If that is possible can you update the documentation and the README.md to reflect the change?
