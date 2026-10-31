import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtemp, mkdir, writeFile, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

const checkerPath = fileURLToPath(new URL("./check-msix-config.mjs", import.meta.url));

async function runChecker(minVersion) {
  const root = await mkdtemp(path.join(os.tmpdir(), "floatly-msix-check-"));
  try {
    await mkdir(path.join(root, "src-tauri"), { recursive: true });
    await mkdir(path.join(root, "packaging/msix"), { recursive: true });

    const pkg = {
      name: "widget-website",
      version: "1.0.0",
      scripts: {
        "build:msix": "powershell -File scripts/build-msix.ps1",
        "check:msix-config": "node scripts/check-msix-config.mjs"
      }
    };
    const packageLock = {
      name: "widget-website",
      version: "1.0.0",
      packages: { "": { name: "widget-website", version: "1.0.0" } }
    };
    const manifest = `<?xml version="1.0"?>
<Package xmlns:uap10="http://schemas.microsoft.com/appx/manifest/uap/windows10/10">
  <Identity Name="__PACKAGE_IDENTITY_NAME__" Publisher="__PACKAGE_PUBLISHER__" Version="__PACKAGE_VERSION__" />
  <Properties><uap10:PackageIntegrity><uap10:Content Enforcement="on" /></uap10:PackageIntegrity></Properties>
  <Dependencies><TargetDeviceFamily Name="Windows.Desktop" MinVersion="${minVersion}" MaxVersionTested="10.0.26100.0" /></Dependencies>
  <Applications><Application uap10:TrustLevel="mediumIL" uap10:RuntimeBehavior="packagedClassicApp" Executable="$targetnametoken$.exe">
    <Extensions><uap5:Extension Category="windows.startupTask"><uap5:StartupTask TaskId="FloatlyStartup" /></uap5:Extension></Extensions>
  </Application></Applications>
  <Capabilities><rescap:Capability Name="runFullTrust" /></Capabilities>
  Assets\\StoreLogo.png Assets\\Square150x150Logo.png Assets\\Square44x44Logo.png Assets\\Wide310x150Logo.png
</Package>`;

    await writeFile(path.join(root, "package.json"), JSON.stringify(pkg));
    await writeFile(path.join(root, "package-lock.json"), JSON.stringify(packageLock));
    await writeFile(path.join(root, "src-tauri/tauri.conf.json"), JSON.stringify({ productName: "Floatly", version: "1.0.0" }));
    await writeFile(path.join(root, "src-tauri/Cargo.toml"), '[package]\nname = "widget-website"\nversion = "1.0.0"\n');
    await writeFile(path.join(root, "src-tauri/Cargo.lock"), '[[package]]\nname = "widget-website"\nversion = "1.0.0"\n');
    await writeFile(path.join(root, "packaging/msix/Package.appxmanifest.template.xml"), manifest);

    return spawnSync(process.execPath, [checkerPath], { cwd: root, encoding: "utf8" });
  } finally {
    await rm(root, { recursive: true, force: true });
  }
}

test("MSIX checker accepts the minimum Windows build required by uap10", async () => {
  const result = await runChecker("10.0.19041.0");
  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
});

test("MSIX checker rejects a Windows minimum below build 19041", async () => {
  const result = await runChecker("10.0.18362.0");
  assert.notEqual(result.status, 0);
  assert.match(`${result.stdout}\n${result.stderr}`, /MinVersion must be 10\.0\.19041\.0 or newer/);
});
