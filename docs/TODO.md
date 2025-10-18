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
- Taking into consideration `docs/technical.md`, `docs/technical/build.md`, `docs/technical/cross-platform-adaptation-plan.md` and `.github/workflows/unified-release.yml` and remembering to only output summaries here, rather than creating new *.md files, can you please ensure that the app doesn't launch with a terminal window. I think logs should be minimal in a production environment and should be sent to the Event Viewer, rather than output in the console.
- Taking into consideration `docs/technical.md`, `docs/technical/build.md`, `docs/technical/cross-platform-adaptation-plan.md` and `.github/workflows/unified-release.yml` and remembering to only output summaries here, rather than creating new *.md files, can you please fix the errors being encountered when trying to install using the MSI on windows:
```
- <Event xmlns="http://schemas.microsoft.com/win/2004/08/events/event">
- <System>
  <Provider Name="MsiInstaller" />
  <EventID Qualifiers="0">10005</EventID>
  <Version>0</Version>
  <Level>2</Level>
  <Task>0</Task>
  <Opcode>0</Opcode>
  <Keywords>0x80000000000000</Keywords>
  <TimeCreated SystemTime="2025-10-18T03:27:03.2045014Z" />
  <EventRecordID>3142</EventRecordID>
  <Correlation />
  <Execution ProcessID="12620" ThreadID="0" />
  <Channel>Application</Channel>
  <Computer>DESKTOP-6GJ1UKP</Computer>
  <Security UserID="S-1-5-21-587828671-3161101140-2418918798-1001" />
  </System>
- <EventData>
  <Data>Product: ZipLock Password Manager -- The installer has encountered an unexpected error installing this package. This may indicate a problem with this package. The error code is 2762. The arguments are: , ,</Data>
  <Data>(NULL)</Data>
  <Data>(NULL)</Data>
  <Data>(NULL)</Data>
  <Data>(NULL)</Data>
  <Data>(NULL)</Data>
  <Binary>7B46434630363735352D444530322D343936322D423645382D4631323135443931363133307D</Binary>
  </EventData>
  </Event>
  ```

## Mac
