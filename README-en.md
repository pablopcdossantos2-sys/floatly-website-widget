# Floatly — Windows

*Leia isto em outros idiomas: [Português](README.md)*

---

**Languages:** [Português (Brasil)](README.md) · English · [Español](README.es.md)

A compact Windows desktop browser that keeps a user-chosen HTTP/HTTPS address visible in a small, frameless WebView2 window. The address is configured on first launch; no third-party website is prefilled. Built with Tauri 2 for Windows 10/11 x64.

## Features

- System tray, single-instance behavior and a global shortcut (default: CTRL+ALT+C). Closing the window hides it in the tray; use **Quit** to exit the app.
- Persistent WebView2 session in %APPDATA%\WidgetWebsite\webview-data; settings in %APPDATA%\WidgetWebsite\config.json. The legacy folder name is kept to preserve data from previous Floatly installations.
- Interface available in Brazilian Portuguese, English and Spanish. Change it under **Settings → Language**.
- A small handle at the top center lets you drag the widget. The edge hit areas remain active but transparent, so no frame is drawn around the content. Window size and position are configurable.
- The default window size is 350 × 550 px and the default embedded browser zoom is 80%; both can be changed in settings.
- Configurable website URL, size, position, movement lock, always-on-top behavior, taskbar visibility, zoom, corner radius and automatic reload.
- External links open in the default browser. Recognized sign-in domains open in a secondary window with a simple address bar and shared WebView2 data folder; the window attempts to close when sign-in returns to the configured website.
- Windows x64 build workflow and NSIS installer.
- An x64 MSIX package for Microsoft Store is built by a dedicated GitHub Actions workflow; see [MICROSOFT_STORE_READINESS.md](MICROSOFT_STORE_READINESS.md) for the publication checklist.

## Requirements

- Windows 10/11 x64, Node.js 22 or newer (LTS) and npm.
- Stable Rust 1.90 or newer.
- Microsoft Edge WebView2 Runtime: https://developer.microsoft.com/microsoft-edge/webview2/. The app checks the usual Runtime locations and displays this address if it cannot find the Runtime.
- To compile locally: Visual Studio Build Tools with Desktop development with C++ and the Windows SDK.

## Development

Run these commands in PowerShell (generating the icons is required before the first build):

    npm ci
    npm run icons
    npm run tauri dev

## Build / package

    npm ci
    npm run icons
    npm run tauri build

The installer is created under src-tauri/target/release/bundle/nsis and the executable under src-tauri/target/release. The Build Windows workflow creates both files and publishes them in the `widget-website-windows-x64` artifact.

## First launch

On first launch, the widget asks you to choose a website. Click **Choose website** and enter a URL under **Settings → Website URL**. The field starts empty: no third-party website is preconfigured. The site must allow loading in WebView2; some sites may restrict specific features or sign-in flows.

For a concrete example URL, see the example section below.

## Microsoft Store

Floatly is prepared for the Store as an **x64 MSIX** package. Follow the [step-by-step guide in Portuguese](GUIA_PUBLICACAO_MICROSOFT_STORE.md) and use the [multilingual listing draft](MICROSOFT_STORE_LISTING_DRAFT.md).

The **Build MSIX for Microsoft Store** workflow runs automatically on `main` when relevant source-code, dependency, configuration or packaging files change. It can also be started manually from the **Actions** tab and uses the package identity already confirmed in Partner Center. The workflow produces an unsigned `.msix` file. This is intentional: Microsoft signs the MSIX after approval. No code-signing certificate, PFX file or signing secrets are required for this route.

## Interface language

Open **Settings → Language** and choose Portuguese (Brazil), English or Spanish. The setting is saved in %APPDATA%\WidgetWebsite\config.json and translates the settings window, system-tray menu, sign-in toolbar and the widget's own connection messages. The chosen website remains in the language served by that website.

## Optional signing for the standalone installer (not used by the Store)

The standalone NSIS installer workflow supports optional Authenticode signing. This is separate from the Microsoft Store MSIX package, which does not use a developer-provided certificate. To sign the NSIS installer for distribution outside the Store, obtain a **trusted Windows code-signing certificate** and create the following repository secrets under **Settings → Secrets and variables → Actions**:

- WINDOWS_PFX_BASE64: the PFX certificate file encoded as Base64.
- WINDOWS_PFX_PASSWORD: the password for that PFX file.

Never put a private key in the repository or this README. The build runs without access to the signing certificate. A separate job uses `signtool` to sign and verify the NSIS installer and the separately published executable with SHA-256 and a timestamp; it removes the temporary PFX file afterward. The separately published executable is signed, but the copy embedded inside the already-packaged installer was bundled unsigned. Without both secrets, the artifacts are published unsigned.

**Important:** a valid signature cannot guarantee that Microsoft Defender SmartScreen will never show a warning. New apps or files may still display a warning until publisher and file reputation are established. Publishing through the Microsoft Store is the most direct way to avoid SmartScreen download warnings. See [Microsoft's SmartScreen reputation guidance](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation) and [Tauri's Windows code-signing guide](https://tauri.app/distribute/sign/windows/).

## Example: Investing.com

Investing.com is only an example of a website the widget can display. To use it, open **Settings → Website URL** and enter `https://br.investing.com/economic-calendar`. The widget is not tied to this service; any website using HTTP or HTTPS can be configured.

For sites that require sign-in, use the authentication flow provided by that website. The system-tray menu includes an option to sign in or configure the website.

The app does not read, copy or store passwords. The default WebView2 user agent is preserved.

## Uninstalling and stored data

The NSIS uninstaller offers a **Delete the application data** checkbox. When selected, it removes `%APPDATA%\\WidgetWebsite` (configuration, logs and WebView2 session data) and `%LOCALAPPDATA%\\WidgetWebsite` if present. If left unchecked, these files are preserved for a later reinstall. The Windows startup entry is always removed when the app is uninstalled.


## 👤 Authorship and development

Floatly is a Windows desktop application independently developed by Pablo Phillipe Cândido dos Santos. It keeps a user-selected HTTP or HTTPS address in a compact floating window powered by Microsoft Edge WebView2. Users can configure the website and adjust window and browsing behavior, with access through the system tray.

Generative AI tools were used as supporting resources during development. The author remains responsible for the project's conception, implementation, integration and verification.

Lattes Curriculum: [http://lattes.cnpq.br/9500873674712528](http://lattes.cnpq.br/9500873674712528)
