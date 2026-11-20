# Kyouzai

A lightweight, extensible desktop companion platform

> Kyouzai - 共在

## Overview

Kyouzai is a desktop companion platform designed to bring interactive companions to your desktop.

The project aims to provide a lightweight foundation for desktop companions, with an extensible architecture that can support future features and behaviors.

---

## Features

- Desktop Companion — Displays an animated 2D companion sprits on the desktop.

- Transparent Window — Renders the companion without a conventional window background or decorations.

- Always on Top — Keeps the companion above other windows.

- Interactive Behavior — Supports mouse interaction with the companion.

- System Tray — Provides basic application controls through the system tray.

- Persistent State — Saves application state across sessions.

- Modular Architecture — Separates application logic, platform integration, asset handling, and rendering into dedicated crates.

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

- Windows 10 or later.

- Rust (stable toolchain).

- Microsoft C++ Build Tools with the MSVC toolchain.

- A graphics device with a supported graphics backend.

### Clone the Repository

```
git clone https://github.com/<username>/kyouzai.git
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
