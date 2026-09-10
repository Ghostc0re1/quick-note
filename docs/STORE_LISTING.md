# Microsoft Store listing and release setup

Scattered Thoughts is a free Windows 11 x64 Microsoft Store application. Its listing
should describe it as a local-first tray note app and link to the repository's
rendered `PRIVACY.md` as its privacy policy.

## Customer-facing listing copy

### Product description

Scattered Thoughts is a quiet place to catch an idea before it disappears.
It lives in the Windows system tray, ready whenever you need it. Press
`Ctrl+Alt+Space`, type a note, and press Enter to save it.

The capture window stays small, focused, and out of the way. Notes are saved
locally on your device in a SQLite database, with no account, cloud sync,
telemetry, or advertising. Open Recent Notes from the tray to revisit the last
100 things you captured.

### What’s new in version 0.1.0

Initial release of Scattered Thoughts: a lightweight Windows tray app for
instant local note capture, recent-note viewing, optional launch at sign-in,
and an optional Keep PC awake toggle.

### Product features

Enter each line below as a separate feature in Partner Center; do not include
the Markdown bullets in the Store form.

- Capture a note anywhere with Ctrl+Alt+Space.
- Save notes locally with no account or cloud service.
- Keep a focused, borderless composer out of your way.
- Review your 100 most recent notes from the system tray.
- Keep an unfinished draft while hiding and reopening the composer.
- Choose whether Scattered Thoughts launches at sign-in.
- Optionally keep the PC and display awake for the current app session.

### Store logo

Use `windows/Assets/StoreLogo300x300.png` as the 1:1 Store logo. It is a
300 by 300 PNG derived from the app’s existing icon.

## One-time Partner Center setup

1. Enroll and verify the developer account, then reserve **Scattered Thoughts**.
2. Create the initial free product, complete category, age rating, listing copy,
   screenshots, Store logo, and certification notes. The product identity must
   match its Partner Center reservation exactly.
3. Copy the Partner Center-assigned values from **Product identity** into GitHub
   repository variables:
   `MSIX_IDENTITY_NAME`, `MSIX_PUBLISHER`, `MSIX_PUBLISHER_DISPLAY_NAME`, and
   `MSSTORE_PRODUCT_ID`.
4. Run the **Build initial Store package** GitHub Actions workflow. Download its
   `initial-store-msix` artifact, then upload the contained MSIX on Partner
   Center's **Packages** page. This workflow only builds an artifact; it does
   not submit anything to Microsoft.
5. Complete the first Store submission manually after Partner Center validates
   the package and the Store listing is ready.
6. Create a Microsoft Entra application, grant it the Partner Center Manager
   role, then add its tenant ID, client ID, client secret, and seller ID as the
   four GitHub Actions secrets named in `release-store.yml`.

The repository variables are identifiers rather than secrets. Keep the Entra
client secret in GitHub Actions secrets, never in a repository variable or file.

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
