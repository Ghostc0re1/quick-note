import "./styles.css";
import { nativeApi } from "./api";
import { attachCaptureController } from "./capture";

const textarea = document.querySelector<HTMLTextAreaElement>("#note");
const status = document.querySelector<HTMLElement>("#status");

if (textarea === null || status === null) {
  throw new Error("Scattered Thoughts capture UI is missing required elements.");
}

attachCaptureController(textarea, status, nativeApi);
