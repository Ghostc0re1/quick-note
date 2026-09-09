import "./styles.css";
import { nativeApi } from "./api";
import { attachHistoryController } from "./history";

const container = document.querySelector<HTMLElement>("#notes");
const status = document.querySelector<HTMLElement>("#history-status");

if (container === null || status === null) {
  throw new Error("Quick Note history UI is missing required elements.");
}

void attachHistoryController(container, status, nativeApi)();
