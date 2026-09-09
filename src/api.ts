import { invoke } from "@tauri-apps/api/core";

export interface Note {
  id: number;
  body: string;
  createdAt: number;
}

export interface QuickNoteApi {
  hideCapture(): Promise<void>;
  listRecentNotes(): Promise<Note[]>;
  saveNote(body: string): Promise<Note>;
}

export const nativeApi: QuickNoteApi = {
  hideCapture: () => invoke<void>("hide_capture"),
  listRecentNotes: () => invoke<Note[]>("list_recent_notes"),
  saveNote: (body) => invoke<Note>("save_note", { body }),
};
