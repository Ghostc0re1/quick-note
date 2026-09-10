import { beforeEach, describe, expect, it, vi } from "vitest";
import { attachHistoryController, type HistoryElements } from "./history";

function elements(): HistoryElements {
  document.body.innerHTML = [
    "<input />",
    "<button>Export</button>",
    "<button>Backup</button>",
    "<p></p>",
    "<div></div>",
    "<dialog><p></p><button>Cancel</button><button>Delete</button></dialog>",
  ].join("");
  const buttons = document.querySelectorAll("button");
  const dialog = document.querySelector("dialog") as HTMLDialogElement;
  return {
    search: document.querySelector("input") as HTMLInputElement,
    exportButton: buttons[0],
    backupButton: buttons[1],
    status: document.querySelector("p") as HTMLParagraphElement,
    container: document.querySelector("div") as HTMLDivElement,
    deleteDialog: dialog,
    deleteMessage: dialog.querySelector("p") as HTMLParagraphElement,
    deleteCancel: dialog.querySelectorAll("button")[0],
    deleteConfirm: dialog.querySelectorAll("button")[1],
  };
}

function api(notes = [{ id: 1, body: "first\nsecond", createdAt: 1_700_000_000, pinned: false }]) {
  return {
    backupDatabase: vi.fn().mockResolvedValue(true),
    copyNote: vi.fn().mockResolvedValue(undefined),
    deleteNote: vi.fn().mockResolvedValue(undefined),
    exportNotes: vi.fn().mockResolvedValue(true),
    listNotes: vi.fn().mockResolvedValue(notes),
    setNotePinned: vi.fn().mockResolvedValue(undefined),
  };
}

describe("history controller", () => {
  beforeEach(() => {
    vi.useRealTimers();
  });

  it("renders note bodies as text and preserves newlines", async () => {
    const ui = elements();
    const load = attachHistoryController(ui, api());
    await load();

    expect(ui.container.querySelector(".note-body")?.textContent).toBe("first\nsecond");
    expect(ui.container.querySelector(".note-body")?.innerHTML).not.toContain("<br>");
  });

  it("shows pinned notes before recent notes", async () => {
    const ui = elements();
    const load = attachHistoryController(
      ui,
      api([
        { id: 1, body: "recent", createdAt: 2, pinned: false },
        { id: 2, body: "pinned", createdAt: 1, pinned: true },
      ]),
    );
    await load();

    expect(ui.container.querySelector("h2")?.textContent).toBe("Pinned");
    expect(ui.container.textContent).toContain("Recent");
  });

  it("searches after a short debounce", async () => {
    vi.useFakeTimers();
    const ui = elements();
    const client = api();
    attachHistoryController(ui, client);
    ui.search.value = "needle";
    ui.search.dispatchEvent(new Event("input"));

    await vi.advanceTimersByTimeAsync(150);
    expect(client.listNotes).toHaveBeenCalledWith("needle");
  });

  it("copies from the overflow menu and confirms before deletion", async () => {
    const ui = elements();
    const client = api();
    const load = attachHistoryController(ui, client);
    await load();

    (ui.container.querySelector('[data-action="copy"]') as HTMLButtonElement).click();
    await vi.waitFor(() => expect(client.copyNote).toHaveBeenCalledWith(1));

    (ui.container.querySelector('[data-action="delete"]') as HTMLButtonElement).click();
    expect(ui.deleteDialog.open).toBe(true);
    expect(client.deleteNote).not.toHaveBeenCalled();

    ui.deleteConfirm.click();
    await vi.waitFor(() => expect(client.deleteNote).toHaveBeenCalledWith(1));
  });

  it("keeps errors visible when an export fails", async () => {
    const ui = elements();
    const client = api();
    client.exportNotes.mockRejectedValueOnce(new Error("Disk unavailable"));
    attachHistoryController(ui, client);

    ui.exportButton.click();
    await vi.waitFor(() => expect(ui.status.textContent).toBe("Disk unavailable"));
  });
});
