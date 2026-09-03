<div align="center">
  <img src="assets/icon.png" width="96" alt="">
</div>

# Winvestour Desktop

The native desktop shell for [Winvestour](https://www.winvestour.com/winvestour) — your whole business in one free app: online store (Wommerce), social media automation (Wocial), influencer (Winfluencers) and reseller (Wellers) programs under one account. Built with [Tauri v2](https://tauri.app): a small, native window (~2 MB) that opens the live Winvestour web app.

**Download the latest release:** see the [Releases](https://github.com/Winvestour/winvestour/releases) page for Windows and Linux builds.

<div align="center">

<a href="https://github.com/Winvestour/winvestour/releases/latest"><img src="https://img.shields.io/github/v/release/Winvestour/winvestour?style=for-the-badge&color=00468C&label=latest" alt="Latest release"></a>
<a href="https://github.com/Winvestour/winvestour/releases"><img src="https://img.shields.io/github/downloads/Winvestour/winvestour/total?style=for-the-badge&color=00468C" alt="Downloads"></a>
<img src="https://img.shields.io/badge/platforms-Windows_%C2%B7_Linux-00468C?style=for-the-badge" alt="Windows · Linux">
<a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-00468C?style=for-the-badge" alt="MIT license"></a>

<a href="https://github.com/Winvestour/winvestour/releases/latest"><img src="https://img.shields.io/badge/Download_for_Windows-.exe_%C2%B7_.msi-00468C?style=for-the-badge&logo=windows&logoColor=white" alt="Download for Windows" height="36"></a>&nbsp;
<a href="https://github.com/Winvestour/winvestour/releases/latest"><img src="https://img.shields.io/badge/Download_for_Linux-.deb_%C2%B7_.rpm_%C2%B7_.AppImage-00468C?style=for-the-badge&logo=linux&logoColor=white" alt="Download for Linux" height="36"></a>

<a href="https://www.winvestour.com/winvestour"><b>Website</b></a> · <a href="https://www.winvestour.com/register"><b>Create a free account</b></a> · <a href="https://github.com/Winvestour"><b>All Winvestour apps</b></a>

<img src="assets/hero.webp" alt="" width="760">

<sub>Your whole business in one free app — store, social media, influencer and reseller programs under one account.</sub>

</div>


## What this is

This shell contains **no application logic** — it's a thin native window around `https://www.winvestour.com`. All of Winvestour's actual functionality (store builder, social media AI, influencer and reseller programs, payments) lives on the web and is identical across platforms; this repo only ships the native wrapper (window chrome, tray behavior, auto-sizing) so Winvestour installs and feels like a real desktop app.

- Branded title bar (frameless window, blue, drag-to-move)
- Single-instance (opening a second time focuses the existing window)
- Window size/position remembered between launches
- No telemetry, no bundled secrets, no local data storage beyond what the browser session already does

## Screenshots

<div align="center">
<img src="assets/desktop-1.webp" alt="Winvestour on desktop" width="520">&nbsp;
<img src="assets/tablet-1.webp" alt="Winvestour on tablet" width="210">
<br><br>
<img src="assets/phone-1.webp" alt="" width="170">&nbsp;
<img src="assets/phone-2.webp" alt="" width="170">&nbsp;
<img src="assets/phone-3.webp" alt="" width="170">
<br><br>
<a href="https://www.winvestour.com/winvestour/screenshots">See all screenshots →</a>
</div>

## Building locally

Prerequisites: [Rust](https://rustup.rs) (stable, MSVC toolchain on Windows), [Node.js](https://nodejs.org) 20+, and platform build tools ([Tauri prerequisites](https://tauri.app/start/prerequisites/)).

```bash
npm install
npm run tauri build
```

Output installers land in `src-tauri/target/release/bundle/`.

## Supported platforms

| Platform | Format | Status |
|---|---|---|
| Windows 10/11 | `.exe` (NSIS), `.msi` | ✅ |
| Linux | `.deb`, `.rpm`, `.AppImage` | ✅ (built via CI) |
| macOS | — | Not planned |

## License

MIT — see [LICENSE](LICENSE).
