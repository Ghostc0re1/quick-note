import type { Note, QuickNoteApi } from "./api";

function formatTimestamp(createdAt: number): string {
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(createdAt * 1_000));
}

function renderNotes(container: HTMLElement, notes: Note[]): void {
  container.replaceChildren();

  if (notes.length === 0) {
    const empty = document.createElement("li");
    empty.className = "note";
    empty.textContent = "No saved notes yet.";
    container.append(empty);
    return;
  }

  for (const note of notes) {
    const item = document.createElement("li");
    item.className = "note";

    const timestamp = document.createElement("time");
    timestamp.dateTime = new Date(note.createdAt * 1_000).toISOString();
    timestamp.textContent = formatTimestamp(note.createdAt);

    const body = document.createElement("p");
    body.className = "note-body";
    body.textContent = note.body;

    item.append(timestamp, body);
    container.append(item);
  }
}

export function attachHistoryController(
  container: HTMLElement,
  status: HTMLElement,
  api: Pick<QuickNoteApi, "listRecentNotes">,
): () => Promise<void> {
  const load = async () => {
    status.textContent = "";
    try {
      renderNotes(container, await api.listRecentNotes());
    } catch (error) {
      container.replaceChildren();
      status.textContent = error instanceof Error ? error.message : "Could not load recent notes.";
    }
  };

  window.addEventListener("focus", () => void load());
  return load;
}
