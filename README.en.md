# Widget Website — Windows

**Languages:** [Português (Brasil)](README.md) · English · [Español](README.es.md)

A small Windows desktop widget that keeps the Investing.com economic calendar open in a compact, frameless WebView2 window. Built with Tauri 2 for Windows 10/11 x64.

## Features

- System tray, single-instance behavior and a global shortcut (default: CTRL+ALT+C). Closing the window hides it in the tray; use **Quit** to exit the app.
- Persistent WebView2 session in %APPDATA%\WidgetWebsite\webview-data; settings in %APPDATA%\WidgetWebsite\config.json.
- Interface available in Brazilian Portuguese, English and Spanish. Change it under **Settings → Language**.
- Move the widget by dragging the visible handle at the top, or hold Alt while dragging any area. Window size and position are also configurable.
- Configurable website URL, size, position, movement lock, always-on-top behavior, taskbar visibility, opacity, zoom, corner radius and automatic reload.
- External links open in the default browser. Recognized sign-in domains open in a secondary window with a simple address bar and shared WebView2 data folder; the window attempts to close when sign-in returns to the configured website.
- Windows x64 build workflow and NSIS installer.

## Requirements

- Windows 10/11 x64, Node.js and npm.
- Stable Rust 1.90 or newer.
- Microsoft Edge WebView2 Runtime: https://developer.microsoft.com/microsoft-edge/webview2/
- To compile locally: Visual Studio Build Tools with Desktop development with C++ and the Windows SDK.

## Development

Run these commands in PowerShell (generating the icons is required before the first build):

    npm install
    npm run icons
    npm run tauri dev

## Build / package

    npm install
    npm run icons
    npm run tauri build

The installer is created under src-tauri/target/release/bundle/nsis and the executable under src-tauri/target/release. GitHub Actions builds the Windows package and publishes it as the widget-website-windows-x64 workflow artifact.

## Interface language

Open **Settings → Language** and choose Portuguese (Brazil), English or Spanish. The setting is saved in %APPDATA%\WidgetWebsite\config.json and translates the settings window, system-tray menu, sign-in toolbar and the widget's own connection messages. The Investing.com content remains in the language served by the website.

## Digital signing and Windows security warnings

The GitHub Actions workflow supports optional Authenticode signing. To enable it, obtain a **trusted Windows code-signing certificate** and create the following repository secrets under **Settings → Secrets and variables → Actions**:

- WINDOWS_PFX_BASE64: the PFX certificate file encoded as Base64.
- WINDOWS_PFX_PASSWORD: the password for that PFX file.

Never put a private key in the repository or in this README. During the build, the workflow imports the certificate, signs with SHA-256 and a timestamp, then removes the temporary PFX file. Without both secrets, CI warns that the artifact is unsigned and continues building.

**Important:** a valid signature cannot guarantee that Microsoft Defender SmartScreen will never show a warning. New apps or files may still display a warning until publisher and file reputation are established. Publishing through the Microsoft Store is the most direct way to avoid SmartScreen download warnings. See [Microsoft's SmartScreen reputation guidance](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation) and [Tauri's Windows code-signing guide](https://tauri.app/distribute/sign/windows/).

## First sign-in

1. Open the widget and wait for the calendar to load.
2. The system-tray menu includes **Sign in / login window**. Email and password sign-in is often the most straightforward option.
3. Restart the app to confirm that the session persists. Some filters may be tied to the account and may need to be set again.
4. If the session expires, use the system-tray sign-in command and authenticate again.

The app does not read, copy or store passwords. The default WebView2 user agent is preserved.

## Freeze diagnostics

- Logs are saved to %APPDATA%\WidgetWebsite\diagnostics.log (usually C:\Users\<user>\AppData\Roaming\WidgetWebsite\diagnostics.log). Log timestamps are in UTC. The previous file is rotated to diagnostics.log.1 at approximately 1 MB.
- The page sends a JavaScript/WebView2 heartbeat every five seconds. After 20 seconds without a heartbeat, the monitor records a warning, checks whether the native window is responsive and looks for descendant msedgewebview2.exe processes. No descendant process is a clue that WebView2 may have exited, not proof of the cause.
- Logs record startup, navigation, initialization failures, drag actions and activity recovery. URLs are logged without query parameters or fragments to reduce the chance of storing session tokens.
- In a GUI Windows build, a terminal may not be available. The log file is the reliable diagnostic source; open it with Notepad after reproducing a freeze.
- These diagnostics cannot prove that an external application caused a freeze. They also cannot write more entries if the process or all its threads are abruptly terminated.

## Privacy and limitations

- No telemetry. The main navigation is restricted to the configured website and recognized authentication resources; other HTTP/HTTPS links are opened in the default browser.
- A corrupt configuration file is replaced with defaults. **Restore default size** only changes the dimensions to 500 × 700 px; other preferences are preserved.
- CI runs automated tests for defaults, normalization, URL validation, legacy language configuration and recovery from corrupt configuration; it also builds the Windows installer.
- The secondary sign-in window shares the WebView2 data folder and has an injected address bar. Navigation to unapproved domains is opened in the default browser. Automatic closing depends on the sign-in provider returning to the configured website; some social sign-in flows may require closing the window manually.
- Zoom uses the native WebView2 zoom control. Opacity and rounded corners use CSS/native clipping; corner appearance may vary on Windows 10.
- Network checks show a custom status panel before the main navigation and monitor connectivity while the site is open. Tauri's default wrapper cannot replace every Chromium navigation failure in every scenario.
- Real sign-in, saved filters, shortcut conflicts, 125%/150% scaling, multiple monitors, DPI changes and memory use still need to be tested on Windows hardware.

## Structure

- src-tauri/src: Rust configuration, window, commands, diagnostics and system tray.
- src: TypeScript bootstrap and settings interface.
- icons/widget.svg: vector source used to generate Windows icons.
