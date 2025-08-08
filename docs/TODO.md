# TODO

- Taking into consideration the contents of all the `ziplock/docs/*.md` files, I'd like to work on the design of the view where the user interacts with the list of credentials. Can you please replace the words "ZipLock Password Manager" with the logo? Can you replace the words in the "Refresh", "Add", "Settings" and "Lock" buttons with relevant icons from our icon library (reference ziplock/docks/design.md for details). Can you also remove the Demo: Connection Error stuff as we don't need that kind of thing in the real app interface. I think the Logo and then the buttons for "Refresh", "Add", "Settings" and "Lock" can all be moved to a left column that is just wide enough to house those items in a vertical tile layout. The larger right column should include the existing search and list of credentials.
- Taking into consideration the contents of all the `ziplock/docs/*.md` files, the github action to check for security vulnerabilities in the dependencies had the following error. Can you please rectify this?

    Fetching advisory database from `https://github.com/RustSec/advisory-db.git`
      Loaded 792 security advisories (from /home/runner/.cargo/advisory-db)
    Updating crates.io index
    Scanning Cargo.lock for vulnerabilities (738 crate dependencies)
Crate:     xcb
Version:   0.8.2
Title:     Multiple soundness issues
Date:      2021-02-04
ID:        RUSTSEC-2021-0019
URL:       https://rustsec.org/advisories/RUSTSEC-2021-0019
Solution:  Upgrade to >=1.0
Dependency tree:
xcb 0.8.2
└── x11-clipboard 0.3.3
    └── clipboard 0.5.0
        └── ziplock-linux 0.1.0
