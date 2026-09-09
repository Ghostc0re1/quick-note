# Quick Note

Quick Note is a Windows 11, local-first desktop app for capturing notes from
the system tray. Press `Ctrl+Alt+Space`, type, and press Enter to save the note
to a local SQLite database.

## Install

Quick Note is distributed through the Microsoft Store. Install it there, launch
it once, and it will run in the system tray. Its **Launch at sign-in** tray
option is off by default and can be changed at any time.

Notes stay on the device. Read the [privacy statement](PRIVACY.md).

## Development

Install [Node.js](https://nodejs.org/), the stable
[Rust toolchain](https://www.rust-lang.org/tools/install), and Microsoft WinApp
CLI (`winget install Microsoft.winappcli --source winget`). Then run:

```powershell
npm install
npm run tauri dev
```

Run checks before handing off work:

```powershell
npm run check
npm run test
npm run rust:fmt
npm run rust:clippy
npm run rust:test
npm run msix:validate
```

`npm run msix:validate` creates an unsigned development MSIX under `artifacts/`.
Only the Store release workflow produces customer builds. Store packages keep
SQLite data in package-local Windows storage; unpackaged development builds use
Tauri's local application-data directory. Neither database is stored here.

See [the detailed v1 design](docs/QUICK_NOTE_V1.md) and
[development baseline](docs/DEVELOPMENT.md) for behavior and project guidance.
See [Store setup and releases](docs/STORE_LISTING.md) before enabling publishing.
