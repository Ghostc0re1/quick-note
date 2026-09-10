import "./styles.css";
import { nativeApi } from "./api";
import { attachHistoryController } from "./history";

const backupButton = document.querySelector<HTMLButtonElement>("#backup-database");
const container = document.querySelector<HTMLElement>("#notes");
const deleteCancel = document.querySelector<HTMLButtonElement>("#cancel-delete");
const deleteConfirm = document.querySelector<HTMLButtonElement>("#confirm-delete");
const deleteDialog = document.querySelector<HTMLDialogElement>("#delete-note-dialog");
const deleteMessage = document.querySelector<HTMLElement>("#delete-note-message");
const exportButton = document.querySelector<HTMLButtonElement>("#export-notes");
const search = document.querySelector<HTMLInputElement>("#note-search");
const status = document.querySelector<HTMLElement>("#history-status");

if (
  backupButton === null ||
  container === null ||
  deleteCancel === null ||
  deleteConfirm === null ||
  deleteDialog === null ||
  deleteMessage === null ||
  exportButton === null ||
  search === null ||
  status === null
) {
  throw new Error("Scattered Thoughts history UI is missing required elements.");
}

void attachHistoryController(
  {
    backupButton,
    container,
    deleteCancel,
    deleteConfirm,
    deleteDialog,
    deleteMessage,
    exportButton,
    search,
    status,
  },
  nativeApi,
)();
