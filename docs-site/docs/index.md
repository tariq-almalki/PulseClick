---
layout: home
hero:
  name: PulseClick
  text: A fast, simple desktop auto-clicker
  tagline: Precise click patterns, configurable start/stop hotkeys, fixed targets, themes, and click-indicator feedback in a clean desktop tool.
  actions:
    - theme: brand
      text: Get started
      link: /getting-started
    - theme: alt
      text: Read the architecture
      link: /architecture
features:
  - icon: ⚡
    title: Responsive by design
    details: Rust keeps the click worker lightweight while the settings window remains responsive.
  - icon: ◉
    title: Clear visual feedback
    details: Color-coded click indicators appear at the actual click point and can be turned off.
  - icon: ⌨
    title: Keyboard-first control
    details: F6 starts or stops by default, the toggle key is configurable, F8 is the emergency stop, and F9 captures a target position.
---

## What PulseClick does

PulseClick automates repetitive mouse clicking while keeping the important controls visible and easy to understand. It supports single, double, triple, quadruple, and custom 5–1,000-click groups, separate burst and repeat timing, a batched turbo mode, configurable start/stop hotkeys, fixed or current-cursor targets, continuous or fixed-count runs, Black and Light themes, and click feedback to confirm activity.

The app is portable: release builds are published as standalone archives for Windows, Linux, and macOS with no installer or runtime setup required. Linux global hotkeys currently require X11, and macOS requires Accessibility/Input Monitoring permission.

## Documentation map

- [Getting started](/getting-started) explains how to run the executable and perform a safe first test.
- [Configuration](/configuration) describes every setting and the recommended workflows.
- [Code architecture](/architecture) documents the Rust modules, worker lifecycle, input path, and animation pipeline.
- [Development](/development) covers local builds, the documentation site, verification, and the future GitHub Pages path.
