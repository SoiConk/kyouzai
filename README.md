# Kyouzai

A lightweight, extensible desktop companion platform

> Kyouzai - 共在

## Overview

Kyouzai is a desktop companion platform designed to bring interactive companions to your desktop.

The project aims to provide a lightweight foundation for desktop companions, with an extensible architecture that can support future features and behaviors.

---

## Features

- Displays a single animated 2D pet directly on the desktop.

- Uses a transparent, borderless, always-on-top window, with application controls accessible through the system tray.

- Supports basic mouse interaction, including waking, grabbing and dragging the companion.

- Provides some simple companion states at the moment, including Idle, Walking, Sleeping, Dragging, and Falling.

---

## Architecture

```
kyouzai
├─ assets
│  ├─ default-pets
│  │  └─ kyouzai
│  │     ├─ pet.json
│  │     └─ spritesheet.png
│  ├─ logo.ico
│  └─ tray-icon.png
├─ Cargo.lock
├─ Cargo.toml
├─ crates
│  ├─ kyouzai-app			# Application entry point, orchestration, and integration of other crates.
│  │
│  ├─ kyouzai-asset			# Asset loading, decoding, validation, and caching.
│  │
│  ├─ kyouzai-core			# Core domain logic and companion behavior, independent of platform and rendering details.
│  │
│  ├─ kyouzai-platform		# Window management, input handling, and operating system integration.
│  │
│  └─ kyouzai-renderer		# GPU resource management and rendering through wgpu.
│
├─ LICENSE
└─ README.md

```

Each crate has a defined responsibility to keep the codebase modular and make individual components easier to maintain and extend.

The application crate acts as the composition root, connecting the core logic with platform services, asset handling, and rendering.

---

## Getting Started

### Prerequisites

- Windows SDK 10 or later.

- Rust stable toolchain.

- Microsoft Visual Studio Build Tools 2022 or later

- A graphics device with a supported graphics backend.

### Clone the Repository

```
https://github.com/SoiConk/kyouzai.git
cd kyouzai
```

### Build & Run

```
cargo build
cargo run
```

---

## Status

Early development. Features and APIs are subject to change.

---

## License

This project is licensed under the MIT License.

---

## Author

**Sói Con**

GitHub: https://github.com/SoiConk

---

## Credits

- Default pet by [Free Game Assets / CraftPix.net](https://free-game-assets.itch.io/free-street-animal-pixel-art-asset-pack)
