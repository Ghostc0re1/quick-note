# Privacy Policy for Scattered Thoughts

**Last updated: September 10, 2026**

## Overview

Your privacy is important. Scattered Thoughts is a local-first Windows note
application designed to capture short notes without an account, cloud service,
or online profile. This policy explains the information the application handles,
how it handles it, and the choices available to you.

This policy applies to the Scattered Thoughts desktop application, including its
Microsoft Store edition. It does not govern Microsoft, Windows, GitHub, or other
services you may use to install, update, or obtain support for the application.
Those services process information under their own privacy terms.

## At a glance

- Your notes and their creation timestamps stay on your Windows device.
- The app has no account, sign-in, cloud sync, telemetry, analytics, ads, or
  network service.
- The app does not send note content or other application data to the developer
  or to third parties.
- The app does not sell or share personal information, including for targeted
  advertising.
- You decide what to write. Avoid placing sensitive information in any app
  unless you are comfortable storing it on the device.
- Optional diagnostic logging stays on the device, is off by default, and never
  includes note text or is sent to the developer.

## Information the application handles

### Information you provide

Scattered Thoughts handles the text you type into a note. A note can contain
personal information if you choose to enter it. When you save a note, the app
also records a UTC creation timestamp so it can list your recent notes in order.

While the capture window is open, an unsaved draft is kept only in the app's
memory. It is not written to disk until you save it, and it is lost when the
app process ends.

### Information the application does not collect automatically

Scattered Thoughts does not automatically access, collect, or transmit your
name, email address, phone number, account credentials, payment information,
contacts, photos, files outside its own app storage, clipboard contents,
microphone, camera, precise location, browser history, advertising identifier,
hardware identifier, or usage history. It does not use cookies, web beacons,
pixels, or similar tracking technologies.

The app uses Windows APIs to display its tray icon and windows, register its
optional global shortcut, and, if you enable it, request that Windows keep the
PC and display awake during the current session. These APIs are used to provide
the feature; Scattered Thoughts does not record or transmit information from
them.
If you explicitly enable diagnostic logging in the tray menu, the app stores
local error-category entries and timestamps to assist troubleshooting. Those
entries exclude note text, search terms, clipboard contents, credentials, and
raw error details. Logs remain on the device, rotate at a bounded size, and are
never transmitted.


## How the application uses information

The application uses saved note text and timestamps only to provide its core
local features: saving, searching, pinning, copying, deleting, exporting, and
showing up to 100 notes at a time. It uses an unsaved draft only to preserve the
text you have not yet saved while the app is running.

Scattered Thoughts does not use your notes or other app data for advertising,
behavioral profiling, personalization, analytics, marketing, research,
training artificial intelligence models, or product-development analysis.

## Where information is stored and how long it remains

Saved notes are stored in a local SQLite database in the app's per-user Windows
application-data location. The Microsoft Store edition uses Windows-managed
local app storage, which is intended to persist across app updates. Development
builds use a separate local application-data location; their notes are not
automatically imported into the Store edition.

Scattered Thoughts does not upload or synchronize notes. You may choose a
location on your device to export notes as Markdown or to create a SQLite backup;
the app writes only to that location after you select it. Saved notes remain in
the local database until deleted in the app or the app's local data is removed
through Windows or the device's storage-management processes. The app does not
provide restore or import in this release, so keep backups where you can manage
them safely.

Unsaved drafts remain only in memory for the current running session. They are
cleared when you successfully save them or when the app process exits.

## Sharing and disclosure

Scattered Thoughts has no servers and no network-based application service. It
does not disclose, sell, rent, trade, or share note content or other app data
with the developer, advertisers, analytics providers, data brokers, service
providers, or other third parties.

If you install or update the app through Microsoft Store, Microsoft may process
your Microsoft account, purchase, device, installation, or update information
under Microsoft's own privacy statement. That information is not collected by
or made available to Scattered Thoughts through the app.

## Security

Notes are stored locally and are not transmitted by the application. Their
protection depends in part on the security of your device, your Windows user
account, and anyone else who can access that account or its local files.

The SQLite database is not separately encrypted by Scattered Thoughts. Use a
strong Windows sign-in method, keep Windows up to date, and do not leave an
unlocked device available to people you do not trust. No local storage method
can guarantee protection against every form of unauthorized access.

## Your choices and controls

You control what you enter into a note and whether to save it. You may close or
hide the capture window without saving; the draft then remains only in memory
while the app continues to run. You can control the optional **Launch at
sign-in** setting through the app's tray menu and Windows Startup Apps controls.
The optional **Keep PC awake** setting is session-only and resets when the app
quits. You can enable or disable local diagnostic logging at any time through
the tray menu; the choice is stored locally.

Because Scattered Thoughts does not receive a copy of your notes or maintain an
online account, the developer cannot access, correct, export, or delete your
notes remotely. You can manage notes in the app, export or back them up to a
location you select, and manage the app's local data using Windows controls.

## Children

Scattered Thoughts does not knowingly collect personal information from anyone,
including children, because it does not transmit application data or operate an
account service. The app is not designed to solicit personal information from
children. A child who types information into a note stores it locally on the
device in the same way as any other user.

## International data transfers

Scattered Thoughts does not transfer application data internationally because
it does not send that data over a network. Microsoft Store, Windows, GitHub, or
other third-party services may process their own information in other locations
according to their respective policies.

## Changes to this policy

If the app's data practices change, this policy will be updated before or when
the relevant change is released. The "Last updated" date at the top identifies
the most recent revision. Material changes will be described in the updated
policy published with the project.

## Contact

For privacy questions about Scattered Thoughts, open an issue at
<https://github.com/Ghostc0re1/quick-note/issues>. Do not include note text,
passwords, or other sensitive information in a public issue.
