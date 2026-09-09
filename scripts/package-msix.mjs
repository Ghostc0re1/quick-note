import {
  cpSync,
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const isStoreBuild = process.env.MSIX_CHANNEL === "store";
const appVersion = JSON.parse(readFileSync(resolve(root, "package.json"), "utf8")).version;
const versionParts = appVersion.match(/^(\d+)\.(\d+)\.(\d+)$/);

if (!versionParts) {
  throw new Error(`MSIX packaging requires a stable MAJOR.MINOR.PATCH version, received ${appVersion}.`);
}

const identity = isStoreBuild
  ? {
      name: requiredEnvironment("MSIX_IDENTITY_NAME"),
      publisher: requiredEnvironment("MSIX_PUBLISHER"),
      publisherDisplayName: requiredEnvironment("MSIX_PUBLISHER_DISPLAY_NAME"),
    }
  : {
      name: "QuickNote.Development",
      publisher: "CN=Quick Note Development",
      publisherDisplayName: "Quick Note Development",
    };
const stagingDirectory = resolve(root, "artifacts", "msix-staging");
const artifactDirectory = resolve(root, "artifacts");
const executable = resolve(root, "src-tauri", "target", "release", "quick-note.exe");
const output = resolve(artifactDirectory, `QuickNote_${versionParts.slice(1).join(".")}.0_x64.msix`);

if (!existsSync(executable)) {
  throw new Error(`Expected Tauri release executable at ${executable}. Run the Tauri build first.`);
}

rmSync(stagingDirectory, { force: true, recursive: true });
mkdirSync(stagingDirectory, { recursive: true });
cpSync(executable, resolve(stagingDirectory, "quick-note.exe"));
cpSync(resolve(root, "windows", "Assets"), resolve(stagingDirectory, "Assets"), { recursive: true });

const manifest = readFileSync(resolve(root, "windows", "Package.appxmanifest.template"), "utf8")
  .replaceAll("{{IDENTITY_NAME}}", identity.name)
  .replaceAll("{{PUBLISHER}}", identity.publisher)
  .replaceAll("{{PUBLISHER_DISPLAY_NAME}}", identity.publisherDisplayName)
  .replaceAll("{{VERSION}}", `${versionParts.slice(1).join(".")}.0`);
writeFileSync(resolve(stagingDirectory, "AppxManifest.xml"), manifest);

const result = spawnSync(
  "winapp",
  ["package", stagingDirectory, "--manifest", resolve(stagingDirectory, "AppxManifest.xml"), "--output", output, "--skip-pri", "--quiet"],
  { cwd: root, shell: process.platform === "win32", stdio: "inherit" },
);

if (result.error) {
  throw result.error;
}
if (result.status !== 0) {
  throw new Error(`winapp package failed with exit code ${result.status}.`);
}

console.log(`Created ${output}`);

function requiredEnvironment(name) {
  const value = process.env[name];
  if (!value) {
    throw new Error(`${name} must be set for a Microsoft Store package.`);
  }
  return value;
}
