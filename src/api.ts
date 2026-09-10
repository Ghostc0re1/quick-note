import { invoke } from "@tauri-apps/api/core";

export interface Note {
  id: number;
  body: string;
  createdAt: number;
  pinned: boolean;
}

export interface AboutInfo {
  version: string;
  diagnosticsEnabled: boolean;
}

export interface QuickNoteApi {
  aboutInfo(): Promise<AboutInfo>;
  backupDatabase(): Promise<boolean>;
  copyNote(id: number): Promise<void>;
  deleteNote(id: number): Promise<void>;
  exportNotes(): Promise<boolean>;
  hideCapture(): Promise<void>;
  listNotes(query: string): Promise<Note[]>;
  openAboutLink(kind: "privacy" | "support"): Promise<void>;
  openDiagnosticsFolder(): Promise<void>;
  saveNote(body: string): Promise<Note>;
  setNotePinned(id: number, pinned: boolean): Promise<void>;
}

export const nativeApi: QuickNoteApi = {
  aboutInfo: () => invoke<AboutInfo>("about_info"),
  backupDatabase: () => invoke<boolean>("backup_database"),
  copyNote: (id) => invoke<void>("copy_note", { id }),
  deleteNote: (id) => invoke<void>("delete_note", { id }),
  exportNotes: () => invoke<boolean>("export_notes"),
  hideCapture: () => invoke<void>("hide_capture"),
  listNotes: (query) => invoke<Note[]>("list_notes", { query }),
  openAboutLink: (kind) => invoke<void>("open_about_link", { kind }),
  openDiagnosticsFolder: () => invoke<void>("open_diagnostics_folder"),
  saveNote: (body) => invoke<Note>("save_note", { body }),
  setNotePinned: (id, pinned) => invoke<void>("set_note_pinned", { id, pinned }),
};
