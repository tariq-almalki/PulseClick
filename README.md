# PulseClick

PulseClick is a fast, keyboard-first desktop auto-clicker built in Rust. It is designed for precise repetitive input while keeping the interface simple, responsive, and easy to verify at a glance.

## Highlights

- Single, double, triple, quadruple, and custom 5–1,000-click bursts.
- Batched turbo mode when the burst gap is set to 0 ms.
- Configurable start/stop hotkey, with F6 as the default.
- F8 emergency stop and F9 target capture.
- Current-cursor or fixed-position targeting.
- Continuous or fixed-cycle operation.
- Black and Light themes.
- Desktop click indicator on Windows, with an in-app preview on every platform.
- Portable release builds for Windows, Linux, and macOS.

## Requirements

- Windows 10 or later, Linux with an X11 desktop session, or macOS.
- Rust stable toolchain.
- macOS users must grant PulseClick Accessibility/Input Monitoring permission before sending input.
- Linux global hotkeys currently require X11; Wayland support is not available for the global-hotkey backend.
- Node.js 18+ only if you want to run the documentation site locally.

## Build and run

```powershell
cargo run --release
```

Create a portable release binary without launching it:

```text
cargo build --release
```

The binary is written to `target/release/pulseclick.exe` on Windows and `target/release/pulseclick` on Linux or macOS.

## Download a release

Tagged releases publish portable archives for Windows x64, Linux x64, Intel macOS, and Apple Silicon macOS on the [GitHub Releases page](https://github.com/tariq-almalki/PulseClick/releases).

Run the tests with:

```powershell
cargo test --all-targets
```

## Local documentation

The documentation site is in `docs-site/` and uses VitePress:

```powershell
cd docs-site
npm install
npm run docs:dev
```

Then open the local address printed by VitePress. The documentation covers getting started, configuration, architecture, verification, and future deployment notes.

## Project layout

```text
src/main.rs              Rust application, input backends, and Windows indicator code
assets/                  Application icon assets
build.rs                 Windows resource embedding
docs-site/docs/          Local VitePress documentation
```

## Responsible use

PulseClick sends real mouse input to the active desktop or selected target. Test it in a safe window first, keep the emergency stop available, and use it only where automation is allowed.

## License

MIT
