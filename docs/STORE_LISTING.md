# Microsoft Store listing and release setup

Quick Note is a free Windows 11 x64 Microsoft Store application. Its listing
should describe it as a local-first tray note app and link to the repository's
rendered `PRIVACY.md` as its privacy policy.

## One-time Partner Center setup

1. Enroll and verify the developer account, then reserve **Quick Note**.
2. Create the initial free product, complete category, age rating, listing copy,
   screenshots, Store logo, and certification notes.
3. Copy the Partner Center-assigned values into repository variables:
   `MSIX_IDENTITY_NAME`, `MSIX_PUBLISHER`, `MSIX_PUBLISHER_DISPLAY_NAME`, and
   `MSSTORE_PRODUCT_ID`.
4. Create a Microsoft Entra application, grant it the Partner Center Manager
   role, then add its tenant ID, client ID, client secret, and seller ID as the
   four GitHub Actions secrets named in `release-store.yml`.
5. Complete the first Store submission manually. Automated tag releases are for
   later package updates after that initial product is live.

## Release procedure

1. Set the same stable `MAJOR.MINOR.PATCH` version in `package.json`,
   `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml`.
2. Run `npm run version:check` and the normal local verification commands.
3. Create and push an annotated tag named `vMAJOR.MINOR.PATCH`.
4. The release workflow validates, packages, uploads a seven-day diagnostic
   artifact, then submits the MSIX to Partner Center. Store certification is
   still the gate before customers receive the update.

GitHub tags are source releases only. Do not attach unsigned installers or
development certificates to GitHub releases.
