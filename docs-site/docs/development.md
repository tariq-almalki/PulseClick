# Development

## Requirements

- Windows, Linux with X11, or macOS
- Rust and Cargo
- Node.js and npm for the documentation site

The mouse backend uses Enigo across platforms. Global hotkeys use the native desktop backend: Windows and macOS are supported, while Linux currently requires X11. macOS development builds need Accessibility/Input Monitoring permission to send input.

## Build the desktop app locally

From the `pulseclick` project directory:

```powershell
cargo fmt -- --check
cargo check
cargo build --release
```

The binary is written to `target/release/pulseclick.exe` on Windows and `target/release/pulseclick` on Linux or macOS.

To check the supported release targets locally when their Rust standard libraries are installed:

```text
cargo check --target x86_64-unknown-linux-gnu
cargo check --target x86_64-apple-darwin
cargo check --target aarch64-apple-darwin
```

## Run the documentation site locally

From `docs-site`:

```powershell
npm install
npm run docs:dev
```

VitePress prints a local address. Open it in a browser to browse the documentation with the default VitePress theme.

To verify the static build:

```powershell
npm run docs:build
npm run docs:preview
```

The generated site is in `docs/.vitepress/dist` and is suitable for a local static server.

## Local-only deployment today

The docs configuration has no external service dependency. Keep the site local with `docs:dev`, or serve the built `docs/.vitepress/dist` directory with any local static server.

## GitHub release builds

The `.github/workflows/release.yml` workflow builds portable archives for Windows x64, Linux x64, Intel macOS, and Apple Silicon macOS when a `v*` tag is pushed. It does not create an installer; each archive contains the application binary, README, and MIT license.

## Practical verification checklist

1. Launch the release binary for the current platform.
2. Start a short fixed-count run in a safe area.
3. Confirm the status changes from IDLE to STARTING to RUNNING.
4. Confirm the click indicator appears and F8 stops the worker.
5. Test F9 capture and fixed-position mode.
6. Run the docs production build before publishing changes.
