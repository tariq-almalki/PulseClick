# Code architecture

PulseClick is a small native Rust application. The UI and worker are intentionally separated so timing and native input calls do not block the settings window.

## Source layout

```text
pulseclick/
├─ assets/
│  ├─ pulseclick.png       # Generated source art
│  ├─ pulseclick-256.png   # Icon artwork used at runtime
│  └─ pulseclick.ico       # Embedded Windows Explorer/taskbar icon resource
├─ src/
│  └─ main.rs              # UI, worker lifecycle, input backends, animated overlay
├─ Cargo.toml              # Rust package and GUI/input/image dependencies
├─ Cargo.lock              # Reproducible dependency resolution
└─ docs-site/
   ├─ docs/                 # VitePress Markdown documentation
   └─ package.json          # Local docs commands
```

## Runtime flow

```text
UI Start / configured toggle key
   │
   ▼
ClickSettings snapshot ──► background click worker
   │                              │
   │                              ├─ start delay
   │                              ├─ target selection
   │                              ├─ native input down/up pairs
   │                              ├─ optional zero-gap batch
   │                              ├─ burst gap inside multi-click group
   │                              └─ Click event (throttled)
   │                                             │
   ▼                                             ▼
UI state ◄── worker event channel ◄──── native click-indicator overlay
```

## Input and hotkey backends

The platform module keeps the worker independent from the operating system:

- Windows uses the native User32 input path and a click-through layered overlay.
- Linux and macOS use the Enigo input backend for cursor position, movement, and button events.
- The global-hotkey backend registers the configurable toggle plus the fixed F8/F9 safety shortcuts on Windows and macOS, and on Linux under X11.
- The controller registers the new toggle key before releasing the previous one, so a failed reassignment does not silently remove the working shortcut.

On Linux, the global hotkey and input release currently require an X11 desktop session. On macOS, the user must grant Accessibility/Input Monitoring permission to allow synthetic input and global shortcuts.

The worker checks an atomic stop flag between operations and during waits. The wait is split into small slices, so F8 remains responsive even when a long interval is configured.

## Worker lifecycle

The UI stores a shared `running` flag, a `starting` flag, and a per-run stop flag. Starting creates a settings snapshot and spawns one worker thread. Stopping sets the flag, joins that worker, and clears the state before returning to idle.

The worker sends one of three completion events: fixed-count completion, safe stop, or native input failure. This lets the UI show a useful final status instead of silently returning to idle.

## Click animation

For a click event, the worker includes the click coordinates and button type. The Windows bridge sends that event to one reusable native layered window with:

- a transparent, click-through surface;
- an always-on-top, non-activating window;
- a 32-bit premultiplied-alpha frame buffer;
- two expanding concentric rings;
- four rotating segmented brackets and an orbit point;
- impact rays and a center pulse.

The overlay is currently implemented for Windows, where it is only visual feedback: it does not receive input and does not change the target application. Reusing one click-through layer prevents high-rate clicking from creating a stack of tiny windows. Linux and macOS use the in-app Preferences preview until native desktop overlays are added for those window systems.

### Visual rationale

The effect uses a short-lived halo, segmented arcs, a moving orbit point, and a center flash so it communicates “the click happened” without covering the target for long. This follows the click-feedback guidance in [Microsoft's Win32 animation guidance](https://github.com/MicrosoftDocs/win32/blob/docs/desktop-src/uxguide/vis-animations.md) and the cursor highlighting pattern used by [PowerToys Mouse Utilities](https://learn.microsoft.com/en-us/windows/powertoys/mouse-utilities), while keeping the PulseClick marker visually distinct. Left, right, and middle clicks use neutral high-contrast shades that adapt to the selected Black or Light theme.

## Icon loading

The clean `assets/pulseclick-256.png` is embedded at compile time. The release window passes its decoded RGBA pixels to egui/winit, so the PulseClick mark is used by the desktop window wherever the platform exposes an application icon. The `.ico` resource is embedded only in Windows builds.
