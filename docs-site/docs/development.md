# Development

## Requirements

- Windows
- Rust and Cargo
- Node.js and npm for the documentation site

## Build the desktop app locally

From the `pulseclick` project directory:

```powershell
cargo fmt -- --check
cargo check
cargo build --release
```

The executable is written to `target/release/pulseclick.exe`.

## Build the MSI installer locally

From the project directory:

```powershell
.\installer\build-msi.ps1
```

The script downloads pinned WiX 4.0.6 packages only when needed, verifies their hashes, builds a compressed x64 MSI, and writes it to `dist/`. Use `-CopyToDownloads` when you want a copy in the current Windows user's Downloads folder.

For release trust, sign both the executable and the MSI with a trusted code-signing certificate:

```powershell
.\installer\sign-release.ps1 -CertificateThumbprint "YOUR_CERTIFICATE_THUMBPRINT"
```

Do not use a self-signed certificate for customer distribution. It does not establish public Windows trust and will not reliably remove the SmartScreen warning.

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

## Later GitHub Pages deployment

When the project gets a GitHub repository, add a GitHub Actions workflow that runs `npm ci` and `npm run docs:build`, then publishes `docs/.vitepress/dist`. For a repository hosted at `https://<user>.github.io/<repo>/`, set VitePress `base` to `/<repo>/` before deployment. The current configuration intentionally leaves `base` at `/` so the local site works without a repository name.

## Practical verification checklist

1. Launch the release executable.
2. Start a short fixed-count run in a safe area.
3. Confirm the status changes from IDLE to STARTING to RUNNING.
4. Confirm the click indicator appears and F8 stops the worker.
5. Test F9 capture and fixed-position mode.
6. Run the docs production build before publishing changes.
