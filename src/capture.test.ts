import { beforeEach, describe, expect, it, vi } from "vitest";
import { attachCaptureController } from "./capture";

const dispatchKey = (element: HTMLElement, key: string, shiftKey = false) =>
  element.dispatchEvent(new KeyboardEvent("keydown", { key, shiftKey, bubbles: true, cancelable: true }));

describe("capture controller", () => {
  let textarea: HTMLTextAreaElement;
  let status: HTMLParagraphElement;
  let hideCapture: ReturnType<typeof vi.fn>;
  let saveNote: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    document.body.innerHTML = "<textarea></textarea><p></p>";
    textarea = document.querySelector("textarea") as HTMLTextAreaElement;
    status = document.querySelector("p") as HTMLParagraphElement;
    hideCapture = vi.fn().mockResolvedValue(undefined);
    saveNote = vi.fn().mockResolvedValue({ id: 1, body: "note", createdAt: 1 });
    attachCaptureController(textarea, status, { hideCapture, saveNote, listRecentNotes: vi.fn() });
  });

  it("saves a note with Enter, then clears and hides capture", async () => {
    textarea.value = "A note";
    dispatchKey(textarea, "Enter");
    await vi.waitFor(() => expect(saveNote).toHaveBeenCalledWith("A note"));
    expect(textarea.value).toBe("");
    expect(hideCapture).toHaveBeenCalledTimes(1);
  });

  it("preserves a draft when Escape hides capture", async () => {
    textarea.value = "unfinished";
    dispatchKey(textarea, "Escape");
    await vi.waitFor(() => expect(hideCapture).toHaveBeenCalledTimes(1));
    expect(textarea.value).toBe("unfinished");
    expect(saveNote).not.toHaveBeenCalled();
  });

  it("hides blank input without saving", async () => {
    textarea.value = "   ";
    dispatchKey(textarea, "Enter");
    await vi.waitFor(() => expect(hideCapture).toHaveBeenCalledTimes(1));
    expect(saveNote).not.toHaveBeenCalled();
  });

  it("does not submit Shift+Enter", () => {
    textarea.value = "multiline";
    dispatchKey(textarea, "Enter", true);
    expect(saveNote).not.toHaveBeenCalled();
    expect(hideCapture).not.toHaveBeenCalled();
  });

  it("shows persistence errors without losing the draft", async () => {
    saveNote.mockRejectedValueOnce(new Error("Disk unavailable"));
    textarea.value = "keep this";
    dispatchKey(textarea, "Enter");
    await vi.waitFor(() => expect(status.textContent).toBe("Disk unavailable"));
    expect(textarea.value).toBe("keep this");
    expect(hideCapture).not.toHaveBeenCalled();
  });
});
