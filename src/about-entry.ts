import "./styles.css";
import { nativeApi } from "./api";

const diagnostics = document.querySelector<HTMLElement>("#about-diagnostics");
const logs = document.querySelector<HTMLButtonElement>("#open-diagnostics");
const privacy = document.querySelector<HTMLButtonElement>("#open-privacy");
const status = document.querySelector<HTMLElement>("#about-status");
const support = document.querySelector<HTMLButtonElement>("#open-support");
const version = document.querySelector<HTMLElement>("#about-version");

if (
  diagnostics === null ||
  logs === null ||
  privacy === null ||
  status === null ||
  support === null ||
  version === null
) {
  throw new Error("Scattered Thoughts About UI is missing required elements.");
}

const showError = (error: unknown) => {
  status.textContent = error instanceof Error ? error.message : "Could not complete that action.";
};

const load = async () => {
  try {
    const info = await nativeApi.aboutInfo();
    version.textContent = "Version " + info.version;
    diagnostics.textContent = info.diagnosticsEnabled
      ? "Diagnostic logging is enabled locally."
      : "Diagnostic logging is off.";
  } catch (error) {
    showError(error);
  }
};

privacy.addEventListener("click", () => {
  void nativeApi.openAboutLink("privacy").catch(showError);
});
support.addEventListener("click", () => {
  void nativeApi.openAboutLink("support").catch(showError);
});
logs.addEventListener("click", () => {
  void nativeApi.openDiagnosticsFolder().catch(showError);
});
window.addEventListener("focus", () => void load());

void load();
