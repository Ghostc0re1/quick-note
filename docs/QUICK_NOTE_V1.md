# Quick Note v1

## Purpose

Quick Note is a Windows-only, local-first tray application for capturing notes
with minimal interaction. It does not connect to a network, create an account,
or start automatically at sign-in unless the user explicitly enables it.

## User interaction

- The app launches hidden and stays in the system tray until the user selects
  **Quit** from the tray menu.
- `Ctrl+Alt+Space` shows the centered capture window. When Windows can report
  the tray icon position, capture lifts from that point into the center over a
  brief 200ms ease-out transition; otherwise it opens centered immediately.
  The tray icon's left click performs the same capture toggle.
- The tray context menu provides **Show Quick Note**, **Hide Quick Note**,
  **Recent Notes**, **Launch at sign-in**, and **Quit**. Launch at sign-in is
  off by default and is available only in the Microsoft Store edition.
- Capture and recent history are mutually exclusive windows. Opening one hides
  the other so the requested window receives focus.

### Capture

The capture window is a small, borderless, non-resizable multiline text box on
a charcoal surface. Enter saves a nonblank note and hides the window.
Shift+Enter adds a newline. Esc, Alt+F4, and tray Hide hide the window without
saving; the unsaved draft stays in memory until the app exits or it is
successfully saved. Enter on blank or whitespace-only text simply hides the
window.

### Recent Notes

Recent Notes opens a separate read-only history window. It shows at most 100
notes ordered newest first, with local display timestamps. Editing, deletion,
search, copying, export, tags, and pagination are intentionally out of scope.

## Implementation

Tauri 2 provides the Windows app shell, tray integration, global shortcut, and
window lifecycle. Rust owns all filesystem and SQLite work. The zero-framework
TypeScript frontend only invokes typed Rust commands.

Notes are stored in SQLite. Store installs use the package's durable Windows
local data folder; unpackaged development builds use Tauri's app-data directory
under `%LOCALAPPDATA%`. The schema is versioned from its first release. Each
note has an integer ID, nonblank text body, and UTC Unix timestamp. The Rust
layer validates note text and the database has the same invariant.

Quick Note is distributed as a signed MSIX through the Microsoft Store for
Windows 11 x64. The Store handles customer updates. The startup task is a
package manifest capability: if Windows disables it in Startup Apps or Task
Manager, Quick Note explains that Windows must re-enable it rather than
overriding the user choice.

If the database cannot initialize, the user receives a native error and the app
does not start with an unusable tray state. If saving fails after startup, the
capture window keeps its draft and displays the error. If the hotkey is already
reserved by another process, tray controls remain available and a native warning
explains that the shortcut could not be registered.

## Verification

Automated coverage verifies migrations, blank-note rejection, exact multiline
storage, timestamp assignment, newest-first history, the 100-note cap, and UI
keyboard behavior. The manual Windows smoke test covers tray actions, hotkey
focus, all hide paths, capture/history exclusivity, persistence after relaunch,
tray-only exit, Store update persistence, and opt-in sign-in startup.
