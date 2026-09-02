# Configuration

## Click pattern

Choose how many physical clicks are sent in each cycle:

| Pattern | Physical clicks per cycle |
| --- | ---: |
| Single | 1 |
| Double | 2 |
| Triple | 3 |
| Quadruple | 4 |
| Custom | 5–1,000 |

Each pattern is one click group. The **Custom** option lets you choose any burst from 5 to 1,000 physical clicks, so 5th, 6th, and higher counts are supported without adding a new preset for every number. The **Burst gap** controls the short spacing inside that group; the **Repeat interval** starts after the group finishes.

## Mouse button

PulseClick can send left, right, or middle mouse button input through the native input backend for the current operating system.

## Timing

**Start delay** is measured in seconds and gives you time to move away from the settings window. It is especially useful when using the current cursor as the target.

The repeat interval is split into hours, minutes, seconds, and milliseconds. A value of zero is normalized to a 1 ms interval to keep the stop path responsive.

For multi-click patterns, **Burst gap** sets the spacing between the physical clicks inside one group. It defaults to 35 ms and is adjustable from 0 to 500 ms. Set it to **0 ms** to submit the entire burst without an intentional pause. A 0 ms gap is fastest, but the actual rate still depends on the target application and system load.

## Target

### Current cursor

The app snapshots the current cursor position at the start of each click group. This keeps a double, triple, or quadruple burst on one target while still following a moving pointer from group to group.

### Fixed screen position

PulseClick moves the cursor to the stored X/Y screen coordinates before each cycle. Press F9 to capture the cursor without needing to click inside the app.

## Repeat

Continuous mode runs until stopped. Fixed-count mode counts cycles, not individual physical clicks. For example, a triple pattern with 20 cycles sends 60 physical clicks; a custom 6-click burst with 20 cycles sends 120.

## Options

- **Keep PulseClick on top** keeps the settings window above other normal windows.
- **Show click indicator** controls the animated desktop feedback.
- **Preview indicator** plays the feedback animation at the current cursor without sending mouse input.

Global shortcuts are registered at launch and the start/stop shortcut can be changed while the app is open:

| Shortcut | Action |
| --- | --- |
| Configured key (F6 by default) | Start or stop |
| F8 | Emergency stop |
| F9 | Capture target position |

F8 and F9 are reserved so the safety and capture paths remain available. If another application owns the configured start/stop key, PulseClick reports that the global hotkey is unavailable; use the on-screen Start/Stop button or choose another key.

Global hotkeys are supported on Windows and macOS, and on Linux when running an X11 desktop session. macOS may require Accessibility/Input Monitoring permission. The current Linux backend does not provide global hotkeys on Wayland.

## Themes

**Black** is the default product theme: near-black graphite surfaces, neutral highlights, and high-contrast text. **Light** uses the same layout with bright surfaces and dark text.
