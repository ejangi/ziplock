# TODO

Run one of the following prompts in Zed's agent panel and when you're ready to bump the version, use `./scripts/version/update-version.sh patch "<My feature update here>"`:

- Taking into consideration `docs/technical.md` and `docs/technical/build.md` and remembering to only output summary information (not create new *.md files), can you please...



## Linux
- Taking into consideration `docs/technical.md` and `docs/architecture.md`, can you ensure the app is creating logs and has the necessary log rotate setup when it's deployed. Not sure if this needs to be in the build or part of the codebase.
- Taking into consideration `docs/technical.md` and `docs/design.md` and remembering to only output summaries here, rather than creating new *.md files, can you please ensure for all credential types that only the title is required? I believe this needs to be done in the `shared/src/models/templates.rs` file.
- Taking into consideration `docs/technical.md` and `docs/design.md` and remembering to only output summaries here, rather than creating new *.md files, can you please add a button under the password field that opens a dialog and helps the user generate a strong password. The dialog should show the proposed password in a plain textbox along with a checkbox to include special characters and a slider to select between 1 and 128 characters. The default should be 24 characters and with the special characters checkbox checked.



## Android
- Taking into consideration `docs/design.md`, `docs/technical.md` and `docs/technical/*.md` and remembering to only output summary information (not create new *.md files), can you please...



## Windows
- Taking into consideration `docs/technical.md`, `docs/technical/build.md`, `docs/technical/cross-platform-adaptation-plan.md` and `.github/workflows/unified-release.yml` and remembering to only output summaries here, rather than creating new *.md files, can you please add some user feedback to the MSI installer so the user knows that the installation succeeded (or failed)? Maybe a little dialogue prompt they have to click "Okay" on.
- Taking into consideration `docs/technical.md`, `docs/technical/build.md`, `docs/technical/cross-platform-adaptation-plan.md` and `.github/workflows/unified-release.yml` and remembering to only output summaries here, rather than creating new *.md files, can you please ensure the Windows *.exe file has the correct icon(s) (original assets are in `assets/icons`). Can you also ensure that the app doesn't launch with a terminal window. I think logs should be minimal in a production environment and should be sent to the Event Viewer, rather than output in the console.
- Taking into consideration `docs/technical.md` and remembering to only output summaries here, rather than creating new *.md files, can you please create a clone of `scripts/dev/run-desktop.sh` for windows cmd.exe so I can quickly compile and run the desktiop app on Windows as part of development?


## Mac
