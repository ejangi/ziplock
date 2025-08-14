# TODO

Run one of the following prompts in Zed's agent panel and when you're ready to bump the version, use `./scripts/version/update-version.sh patch "<My feature update here>"`:

- Taking into consideration the `ziplock/docs/technical.md` file, should we look at making the IpcClient a shared component with C-bindings for use in other languages? I know that shared/src/ffi.rs exists, but I'm unsure whether it includes all the IpcClient functionality. If not, we might need to add it.
- Taking into consideration `docs/design.md`, can you please use `frontend/linux/resources/icons/xmark.svg` as the close icon to for the button that dismisses a toast notification?
- Taking into consideration `docs/design.md`, can you please change the design of the shared toast component? Can you make sure it's container has no padding, that the toast has no margin, that the toast expands to fill the full width of the container and that the toast has no border radius?
- Taking into consideration the `docs/*.md` files, can you please create a helper method in the shared library that can translate from machine-friendly-names like "credit_card" to "Credit Card" and "secure_note" to "Secure Note"? Then can you apply this helper to the button text when listing credential types?
- Taking into consideration the `docs/02-specification.md` file, can you please ensure that all Credential Types have been implemented?
- Taking into consideration `docs/design.md`, can you please ensure that buttons that use our brand purple as a background color use white as their text color?
- Taking into consideration `docs/design.md`, I want to replace the title of the add_credential input page from being "Add New Credential" to just having the Title input field. However I want the title input field to have a larger font that other inputs and also have larger padding that other inputs. That way it stands out more. Similarly, on the edit credential input page, I want the replace the title "Edit: <Credential Title>" with the same title input field.
