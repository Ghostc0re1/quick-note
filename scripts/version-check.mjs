import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const nodeVersion = JSON.parse(readFileSync(resolve(root, "package.json"), "utf8")).version;
const tauriVersion = JSON.parse(readFileSync(resolve(root, "src-tauri", "tauri.conf.json"), "utf8")).version;
const cargoToml = readFileSync(resolve(root, "src-tauri", "Cargo.toml"), "utf8");
const cargoVersion = cargoToml.match(/^version\s*=\s*"([^"]+)"/m)?.[1];

if (!cargoVersion || new Set([nodeVersion, tauriVersion, cargoVersion]).size !== 1) {
  throw new Error(
    `Version mismatch: package.json=${nodeVersion}, tauri.conf.json=${tauriVersion}, Cargo.toml=${cargoVersion ?? "missing"}.`,
  );
}

const releaseTagIndex = process.argv.indexOf("--tag");
const releaseTag = releaseTagIndex === -1 ? undefined : process.argv[releaseTagIndex + 1];
if (releaseTag) {
  if (!/^v\d+\.\d+\.\d+$/.test(releaseTag)) {
    throw new Error(`Release tags must use vMAJOR.MINOR.PATCH, received ${releaseTag}.`);
  }
  if (releaseTag.slice(1) !== nodeVersion) {
    throw new Error(`Release tag ${releaseTag} does not match application version ${nodeVersion}.`);
  }
}

const parts = nodeVersion.match(/^(\d+)\.(\d+)\.(\d+)$/);
if (!parts || parts.slice(1).some((part) => Number(part) > 65535)) {
  throw new Error(`Version ${nodeVersion} cannot become a valid four-part MSIX version.`);
}

console.log(`${nodeVersion} -> ${parts.slice(1).join(".")}.0`);
