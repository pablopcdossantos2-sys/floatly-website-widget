# Widget Website — Windows

**Languages:** [Português (Brasil)](README.md) · English · [Español](README.es.md)

A small Windows desktop widget that keeps a website chosen by the user visible in a compact, frameless WebView2 window. The website is configured on first launch; no third-party URL is prefilled. Built with Tauri 2 for Windows 10/11 x64.

## Features

- System tray, single-instance behavior and a global shortcut (default: CTRL+ALT+C). Closing the window hides it in the tray; use **Quit** to exit the app.
- Persistent WebView2 session in %APPDATA%\WidgetWebsite\webview-data; settings in %APPDATA%\WidgetWebsite\config.json.
- Interface available in Brazilian Portuguese, English and Spanish. Change it under **Settings → Language**.
- A small handle at the top center lets you drag the widget. The edge hit areas remain active but transparent, so no frame is drawn around the content. Window size and position are configurable.
- The default window size is 350 × 550 px and the default embedded browser zoom is 80%; both can be changed in settings.
- Configurable website URL, size, position, movement lock, always-on-top behavior, taskbar visibility, zoom, corner radius and automatic reload.
- External links open in the default browser. Recognized sign-in domains open in a secondary window with a simple address bar and shared WebView2 data folder; the window attempts to close when sign-in returns to the configured website.
- Windows x64 build workflow and NSIS installer.
- Initial Microsoft Store preparation is documented in [MICROSOFT_STORE_READINESS.md](MICROSOFT_STORE_READINESS.md) (Portuguese); submission still depends on the listed outstanding work.

## Requirements

- Windows 10/11 x64, Node.js 22 or newer (LTS) and npm.
- Stable Rust 1.90 or newer.
- Microsoft Edge WebView2 Runtime: https://developer.microsoft.com/microsoft-edge/webview2/
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

The installer is created under src-tauri/target/release/bundle/nsis and the executable under src-tauri/target/release. GitHub Actions builds the Windows package and publishes it as the widget-website-windows-x64 workflow artifact.

## First launch

On first launch, the widget asks you to choose a website. Click **Choose website** and enter a URL under **Settings → Website URL**. The field starts empty: no third-party website is preconfigured. The site must allow loading in WebView2; some sites may restrict specific features or sign-in flows.

For a concrete example URL, see the example section below.

## Interface language

Open **Settings → Language** and choose Portuguese (Brazil), English or Spanish. The setting is saved in %APPDATA%\WidgetWebsite\config.json and translates the settings window, system-tray menu, sign-in toolbar and the widget's own connection messages. The chosen website remains in the language served by that website.

## Digital signing and Windows security warnings

The GitHub Actions workflow supports optional Authenticode signing. To enable it, obtain a **trusted Windows code-signing certificate** and create the following repository secrets under **Settings → Secrets and variables → Actions**:

- WINDOWS_PFX_BASE64: the PFX certificate file encoded as Base64.
- WINDOWS_PFX_PASSWORD: the password for that PFX file.

Never put a private key in the repository or this README. The build runs without access to the signing certificate. A separate job uses `signtool` to sign and verify the NSIS installer and the separately published executable with SHA-256 and a timestamp; it removes the temporary PFX file afterward. The separately published executable is signed, but the copy embedded inside the already-packaged installer was bundled unsigned. Without both secrets, the artifacts are published unsigned.

**Important:** a valid signature cannot guarantee that Microsoft Defender SmartScreen will never show a warning. New apps or files may still display a warning until publisher and file reputation are established. Publishing through the Microsoft Store is the most direct way to avoid SmartScreen download warnings. See [Microsoft's SmartScreen reputation guidance](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation) and [Tauri's Windows code-signing guide](https://tauri.app/distribute/sign/windows/).

## Example: Investing.com

Investing.com is only an example of a website the widget can display. To use it, open **Settings → Website URL** and enter `https://br.investing.com/economic-calendar`. The widget is not tied to this service; any website using HTTP or HTTPS can be configured.

For sites that require sign-in, use the authentication flow provided by that website. The system-tray menu includes an option to sign in or configure the website.

The app does not read, copy or store passwords. The default WebView2 user agent is preserved.

## Freeze diagnostics

- Logs are saved to %APPDATA%\WidgetWebsite\diagnostics.log (usually C:\Users\<user>\AppData\Roaming\WidgetWebsite\diagnostics.log). Log timestamps are in UTC. The previous file is rotated to diagnostics.log.1 at approximately 1 MB.
- The page sends a JavaScript/WebView2 heartbeat every five seconds. After 20 seconds without a heartbeat, the monitor records a warning, checks whether the native window is responsive and looks for descendant msedgewebview2.exe processes. No descendant process is a clue that WebView2 may have exited, not proof of the cause.
- Logs record startup, navigation, initialization failures, drag actions and activity recovery. URLs are logged without query parameters or fragments to reduce the chance of storing session tokens.
- In a GUI Windows build, a terminal may not be available. The log file is the reliable diagnostic source; open it with Notepad after reproducing a freeze.
- These diagnostics cannot prove that an external application caused a freeze. They also cannot write more entries if the process or all its threads are abruptly terminated.

## Privacy and limitations

- No telemetry. Main navigation is restricted to the configured website and recognized authentication resources; links to other domains open in the default browser. The remote website is granted only the authenticated activity-heartbeat command.
- A corrupt configuration file is replaced with defaults. **Restore default size** only changes the dimensions to 350 × 550 px; other preferences are preserved.
- CI checks Tauri version alignment, the command manifest and capabilities; it runs `npm ci`, type checking, `npm audit`, `cargo fmt --check`, `cargo clippy`, `cargo audit` and `cargo test`, then builds the Windows installer.
- The secondary sign-in window shares the WebView2 data folder and has an injected address bar. Navigation to unapproved domains is opened in the default browser. Automatic closing depends on the sign-in provider returning to the configured website; some social sign-in flows may require closing the window manually.
- Zoom uses the native WebView2 zoom control. Corner clipping may vary on Windows 10.
- Network checks show a custom status panel before the main navigation and monitor connectivity while the site is open. Tauri's default wrapper cannot replace every Chromium navigation failure in every scenario.
- Real sign-in, saved filters, shortcut conflicts, 125%/150% scaling, multiple monitors, DPI changes and memory use still need to be tested on Windows hardware.

## Uninstalling and stored data

The NSIS uninstaller offers a **Delete the application data** checkbox. When selected, it removes `%APPDATA%\\WidgetWebsite` (configuration, logs and WebView2 session data) and `%LOCALAPPDATA%\\WidgetWebsite` if present. If left unchecked, these files are preserved for a later reinstall. The Windows startup entry is always removed when the app is uninstalled.

## Structure

- src-tauri/src: Rust configuration, window, commands, diagnostics and system tray.
- src: TypeScript bootstrap and settings interface.
- icons/widget.svg: vector source used to generate Windows icons.
