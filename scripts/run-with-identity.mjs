import { cpSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const stagingDirectory = resolve(root, "artifacts", "identity-dev");
const executable = resolve(root, "src-tauri", "target", "debug", "quick-note.exe");

if (!existsSync(executable)) {
  throw new Error(`Expected debug executable at ${executable}.`);
}

rmSync(stagingDirectory, { force: true, recursive: true });
mkdirSync(stagingDirectory, { recursive: true });
cpSync(executable, resolve(stagingDirectory, "quick-note.exe"));
cpSync(resolve(root, "windows", "Assets"), resolve(stagingDirectory, "Assets"), { recursive: true });
const version = JSON.parse(readFileSync(resolve(root, "package.json"), "utf8")).version;
const versionParts = version.match(/^(\d+)\.(\d+)\.(\d+)$/);
if (!versionParts) {
  throw new Error(`Package-identity development requires a stable MAJOR.MINOR.PATCH version, received ${version}.`);
}
const manifest = readFileSync(resolve(root, "windows", "Package.appxmanifest.template"), "utf8")
  .replaceAll("{{IDENTITY_NAME}}", "QuickNote.Development")
  .replaceAll("{{PUBLISHER}}", "CN=Quick Note Development")
  .replaceAll("{{PUBLISHER_DISPLAY_NAME}}", "Quick Note Development")
  .replaceAll("{{VERSION}}", `${versionParts.slice(1).join(".")}.0`);
writeFileSync(resolve(stagingDirectory, "AppxManifest.xml"), manifest);

const result = spawnSync("winapp", ["run", stagingDirectory], {
  cwd: root,
  shell: process.platform === "win32",
  stdio: "inherit",
});
if (result.error) throw result.error;
if (result.status !== 0) throw new Error(`winapp run failed with exit code ${result.status}.`);
