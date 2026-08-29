# PulseClick

PulseClick is a fast, keyboard-first Windows auto-clicker built in Rust. It is designed for precise repetitive input while keeping the interface simple, responsive, and easy to verify at a glance.

## Highlights

- Single, double, triple, quadruple, and custom 5–1,000-click bursts.
- Batched turbo mode when the burst gap is set to 0 ms.
- Configurable start/stop hotkey, with F6 as the default.
- F8 emergency stop and F9 target capture.
- Current-cursor or fixed-position targeting.
- Continuous or fixed-cycle operation.
- Black and Light themes.
- Desktop click indicator with an in-app preview.
- Portable Windows release executable with an embedded black icon.

## Requirements

- Windows 10 or later.
- Rust stable toolchain with the MSVC target.
- Node.js 18+ only if you want to run the documentation site locally.

## Build and run

```powershell
cargo run --release
```

Create the portable executable without launching it:

```powershell
cargo build --release
```

The executable is written to `target/release/pulseclick.exe`.

### Certificate purpose matters

Let’s Encrypt certificates are Domain Validation TLS certificates for HTTPS websites. They prove control of a domain but are not issued for Windows Authenticode code signing, so they cannot remove the SmartScreen publisher warning for PulseClick. Use Microsoft Artifact Signing with a **Public Trust** profile, or another publicly trusted code-signing certificate, for the EXE and MSI.

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
src/main.rs              Rust application and Windows input/indicator code
assets/                  Application icon assets
build.rs                 Windows resource embedding
docs-site/docs/          Local VitePress documentation
```

## Responsible use

PulseClick sends real mouse input to the active desktop or selected target. Test it in a safe window first, keep the emergency stop available, and use it only where automation is allowed.

## License

MIT
