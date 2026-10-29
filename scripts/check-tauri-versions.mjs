import fs from "node:fs";
const packageLock = JSON.parse(fs.readFileSync("package-lock.json", "utf8"));
const cargoLock = fs.readFileSync("src-tauri/Cargo.lock", "utf8");
const npmVersions = { api: packageLock.packages?.["node_modules/@tauri-apps/api"]?.version, cli: packageLock.packages?.["node_modules/@tauri-apps/cli"]?.version };
function cargoVersion(name) {
  const blocks = cargoLock.split("[[package]]");
  for (const block of blocks) {
    const packageName = /^\s*name = "([^"]+)"/m.exec(block)?.[1];
    if (packageName === name) return /^\s*version = "([^"]+)"/m.exec(block)?.[1];
  }
  return undefined;
}
const rustTauri = cargoVersion("tauri");
const rustBuild = cargoVersion("tauri-build");
const majorMinor = (version) => version?.split(".").slice(0, 2).join(".");
if (!npmVersions.api || !npmVersions.cli || !rustTauri || !rustBuild) throw new Error("Could not locate the Tauri versions in the lockfiles.");
const minors = [majorMinor(npmVersions.api), majorMinor(npmVersions.cli), majorMinor(rustTauri)];
if (new Set(minors).size !== 1) throw new Error("Tauri JS API, CLI and Rust crate are not on the same major/minor: " + JSON.stringify({ npmVersions, rustTauri, rustBuild }));
console.log("Tauri versions align: " + JSON.stringify({ npmVersions, rustTauri, rustBuild }));
