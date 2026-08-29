# Getting started

## Run the app

For the professional installation, run `PulseClick-Setup-<version>-x64.msi`. It installs PulseClick under Program Files, creates a Start Menu shortcut, and registers a clean Windows uninstall entry. A portable `PulseClick.exe` is also available when you do not want an installation.

The first public release should be signed with a trusted code-signing certificate. Without that signature, Windows Defender SmartScreen may show **Unknown publisher** for either the installer or the portable executable.

The first screen opens in an idle state. Configure the pattern and target, then use the large **Start clicking** button or press the configured start/stop key. The default is **F6**.

## Safe first test

1. Leave the click pattern as **Single** and the mouse button as **Left**.
2. Keep the default **2-second start delay**.
3. Select **Use current cursor position**.
4. Press **Start clicking**.
5. Move the cursor to a safe, empty area before the countdown finishes.
6. Press **F8** to stop.

The start delay exists specifically to prevent the Start button from being clicked when the current-cursor mode is used.

## Capture a fixed target

Move the cursor over the exact target, then press **F9**. PulseClick switches to fixed-position mode and stores the screen coordinates. You can also edit the X and Y values directly.

Fixed-position mode moves the cursor to the stored coordinates before clicking. Current-cursor mode follows the cursor at the moment of each click.

## Choose a run plan

- **Until I stop it** keeps clicking until F8, the configured start/stop key, or the Stop button is used.
- **Fixed number of cycles** stops after the configured number of click groups and reports the completed count.

One cycle contains the selected burst. The preset buttons cover single through quadruple clicks; choose **Custom** for 5, 6, or any value up to 1,000 physical clicks per burst.

For the fastest burst, set **Burst gap** to **0 ms**. PulseClick batches the burst through Windows so the input worker does not pause between each physical click. The useful speed limit is usually the target application, not the number in the burst field.

## Visual feedback

The click indicator is enabled by default. It appears as a short, click-through animation at the click location:

- A neutral high-contrast marker for left clicks
- A softer silver marker for right clicks
- A muted gray marker for middle clicks

Use **Preview indicator** in Preferences to play the effect without sending a click. Use **Show click indicator** to disable it. The animation is throttled at very high click rates and reuses one native overlay, so it does not create unnecessary windows or slow down the click worker.

## Choose a start/stop key

Open **Preferences** and choose a key from the **Start / stop hotkey** menu, or select **Record key** and press the key you want. The same key always starts and stops the worker. F8 remains the emergency stop and F9 remains target capture.

## Switch themes

The default **Black** theme uses neutral graphite surfaces and high-contrast text. Choose **Light** in Preferences when you prefer a bright workspace. The selected theme applies immediately to the whole settings window.
