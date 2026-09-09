import type { QuickNoteApi } from "./api";

export function attachCaptureController(
  textarea: HTMLTextAreaElement,
  status: HTMLElement,
  api: QuickNoteApi,
): void {
  const focusComposer = () => {
    textarea.focus();
    textarea.setSelectionRange(textarea.value.length, textarea.value.length);
  };

  const hide = async () => {
    status.textContent = "";
    await api.hideCapture();
  };

  textarea.addEventListener("keydown", async (event) => {
    if (event.key === "Escape") {
      event.preventDefault();
      await hide();
      return;
    }

    if (event.key !== "Enter" || event.shiftKey || event.isComposing) {
      return;
    }

    event.preventDefault();
    if (textarea.value.trim().length === 0) {
      await hide();
      return;
    }

    textarea.disabled = true;
    status.textContent = "";
    try {
      await api.saveNote(textarea.value);
      textarea.value = "";
      await hide();
    } catch (error) {
      status.textContent = error instanceof Error ? error.message : "Could not save note.";
    } finally {
      textarea.disabled = false;
      focusComposer();
    }
  });

  window.addEventListener("focus", focusComposer);
  focusComposer();
}
