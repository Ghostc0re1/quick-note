import type { Note, QuickNoteApi } from "./api";

type HistoryApi = Pick<
  QuickNoteApi,
  "backupDatabase" | "copyNote" | "deleteNote" | "exportNotes" | "listNotes" | "setNotePinned"
>;

export interface HistoryElements {
  backupButton: HTMLButtonElement;
  container: HTMLElement;
  deleteCancel: HTMLButtonElement;
  deleteConfirm: HTMLButtonElement;
  deleteDialog: HTMLDialogElement;
  deleteMessage: HTMLElement;
  exportButton: HTMLButtonElement;
  search: HTMLInputElement;
  status: HTMLElement;
}

function formatTimestamp(createdAt: number): string {
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(createdAt * 1_000));
}

function createButton(label: string, action: string, note: Note): HTMLButtonElement {
  const button = document.createElement("button");
  button.type = "button";
  button.className = action === "pin" ? "note-pin" : "note-action";
  button.dataset.action = action;
  button.dataset.noteId = String(note.id);
  button.setAttribute("aria-label", label);
  button.title = label;
  button.textContent = action === "pin" ? (note.pinned ? "Unpin" : "Pin") : label;
  return button;
}

function createNote(note: Note): HTMLElement {
  const item = document.createElement("article");
  item.className = "note";
  item.dataset.noteId = String(note.id);

  const meta = document.createElement("div");
  meta.className = "note-meta";
  const timestamp = document.createElement("time");
  timestamp.dateTime = new Date(note.createdAt * 1_000).toISOString();
  timestamp.textContent = formatTimestamp(note.createdAt);
  meta.append(timestamp, createButton(note.pinned ? "Unpin note" : "Pin note", "pin", note));

  const body = document.createElement("p");
  body.className = "note-body";
  body.textContent = note.body;

  const menu = document.createElement("details");
  menu.className = "note-menu";
  const summary = document.createElement("summary");
  summary.setAttribute("aria-label", "Note actions");
  summary.textContent = "More";
  const actions = document.createElement("div");
  actions.className = "note-menu-actions";
  actions.append(
    createButton("Copy", "copy", note),
    createButton("Delete", "delete", note),
  );
  menu.append(summary, actions);

  item.append(meta, body, menu);
  return item;
}

function appendSection(container: HTMLElement, title: string, notes: Note[]): void {
  if (notes.length === 0) {
    return;
  }
  const section = document.createElement("section");
  section.className = "notes-section";
  const heading = document.createElement("h2");
  heading.textContent = title;
  const list = document.createElement("div");
  list.className = "notes-list";
  notes.forEach((note) => list.append(createNote(note)));
  section.append(heading, list);
  container.append(section);
}

function renderNotes(container: HTMLElement, notes: Note[]): void {
  container.replaceChildren();

  if (notes.length === 0) {
    const empty = document.createElement("p");
    empty.className = "note empty-note";
    empty.textContent = "No saved notes found.";
    container.append(empty);
    return;
  }

  appendSection(container, "Pinned", notes.filter((note) => note.pinned));
  appendSection(container, "Recent", notes.filter((note) => !note.pinned));
}

function asNoteId(button: HTMLButtonElement): number | undefined {
  const value = Number(button.dataset.noteId);
  return Number.isSafeInteger(value) && value > 0 ? value : undefined;
}

export function attachHistoryController(
  elements: HistoryElements,
  api: HistoryApi,
): () => Promise<void> {
  let selectedForDeletion: number | undefined;
  let searchTimer: number | undefined;
  let requestId = 0;

  const load = async () => {
    const currentRequest = ++requestId;
    elements.status.textContent = "";
    try {
      const notes = await api.listNotes(elements.search.value);
      if (currentRequest === requestId) {
        renderNotes(elements.container, notes);
      }
    } catch (error) {
      if (currentRequest === requestId) {
        elements.container.replaceChildren();
        elements.status.textContent =
          error instanceof Error ? error.message : "Could not load notes.";
      }
    }
  };

  const runAction = async (button: HTMLButtonElement) => {
    const noteId = asNoteId(button);
    if (noteId === undefined) {
      return;
    }
    const action = button.dataset.action;
    button.disabled = true;
    try {
      if (action === "pin") {
        await api.setNotePinned(noteId, button.textContent === "Pin");
        await load();
      } else if (action === "copy") {
        await api.copyNote(noteId);
        elements.status.textContent = "Note copied.";
      } else if (action === "delete") {
        selectedForDeletion = noteId;
        elements.deleteMessage.textContent = "Delete this note permanently?";
        elements.deleteDialog.open = true;
      }
    } catch (error) {
      elements.status.textContent =
        error instanceof Error ? error.message : "Could not update the note.";
    } finally {
      button.disabled = false;
    }
  };

  elements.search.addEventListener("input", () => {
    if (searchTimer !== undefined) {
      window.clearTimeout(searchTimer);
    }
    searchTimer = window.setTimeout(() => void load(), 150);
  });

  elements.container.addEventListener("click", (event) => {
    const button = (event.target as HTMLElement).closest<HTMLButtonElement>("button[data-action]");
    if (button !== null) {
      void runAction(button);
    }
  });

  elements.deleteCancel.addEventListener("click", () => {
    selectedForDeletion = undefined;
    elements.deleteDialog.open = false;
  });

  elements.deleteConfirm.addEventListener("click", async () => {
    if (selectedForDeletion === undefined) {
      return;
    }
    elements.deleteConfirm.disabled = true;
    try {
      await api.deleteNote(selectedForDeletion);
      elements.deleteDialog.open = false;
      selectedForDeletion = undefined;
      elements.status.textContent = "Note deleted.";
      await load();
    } catch (error) {
      elements.status.textContent =
        error instanceof Error ? error.message : "Could not delete the note.";
    } finally {
      elements.deleteConfirm.disabled = false;
    }
  });

  const runFileAction = async (
    button: HTMLButtonElement,
    action: () => Promise<boolean>,
    completed: string,
  ) => {
    button.disabled = true;
    elements.status.textContent = "";
    try {
      elements.status.textContent = (await action()) ? completed : "Canceled.";
    } catch (error) {
      elements.status.textContent =
        error instanceof Error ? error.message : "Could not complete that action.";
    } finally {
      button.disabled = false;
    }
  };

  elements.exportButton.addEventListener("click", () => {
    void runFileAction(elements.exportButton, () => api.exportNotes(), "Notes exported.");
  });
  elements.backupButton.addEventListener("click", () => {
    void runFileAction(elements.backupButton, () => api.backupDatabase(), "Backup created.");
  });
  window.addEventListener("focus", () => void load());

  return load;
}
