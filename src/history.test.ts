import { beforeEach, describe, expect, it, vi } from "vitest";
import { attachHistoryController } from "./history";

describe("history controller", () => {
  beforeEach(() => {
    document.body.innerHTML = "<ol></ol><p></p>";
  });

  it("renders note bodies as text and preserves newlines", async () => {
    const container = document.querySelector("ol") as HTMLOListElement;
    const status = document.querySelector("p") as HTMLParagraphElement;
    const load = attachHistoryController(container, status, {
      listRecentNotes: vi.fn().mockResolvedValue([{ id: 1, body: "first\nsecond", createdAt: 1_700_000_000 }]),
    });

    await load();
    expect(container.querySelector(".note-body")?.textContent).toBe("first\nsecond");
    expect(container.querySelector(".note-body")?.innerHTML).not.toContain("<br>");
  });

  it("shows an empty-state message", async () => {
    const container = document.querySelector("ol") as HTMLOListElement;
    const status = document.querySelector("p") as HTMLParagraphElement;
    const load = attachHistoryController(container, status, { listRecentNotes: vi.fn().mockResolvedValue([]) });

    await load();
    expect(container.textContent).toBe("No saved notes yet.");
  });
});
