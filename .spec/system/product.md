# Product Context

## What This Tool Does

spice2x-cli is a command-line interface for controlling a running instance of spice2x while a Bemani arcade game is being played. It sends inputs to the game (buttons, coins, cards, touch), reads outputs (lights, analogs, game state), and captures screenshots — all over a TCP network protocol called SpiceAPI.

The primary use case is enabling an AI agent to autonomously interact with and observe a Bemani game for testing purposes: verifying IO emulation, network functionality, reverse engineering, or any task requiring programmatic control of game state.

## Who Uses It

- AI agents that need to autonomously test game features
- Developers working on spice2x, Bemani game mods, or arcade hardware emulation
- Anyone who needs scriptable control of a running Bemani game instance

## Domain Concepts

- **spice2x**: A Windows tool that hooks into Bemani arcade games to provide IO emulation, network services, and other functionality. It exposes a SpiceAPI server for remote control.
- **SpiceAPI**: A JSON-over-TCP protocol with null-byte framing and optional RC4 encryption. The protocol uses request/response pairs identified by monotonic IDs.
- **Bemani**: Konami's music game franchise (beatmania IIDX, Dance Dance Revolution, Sound Voltex, etc.). Each game has specific IO hardware (buttons, turntables, dance pads, touch screens).
- **Session refresh**: On connect, the client sends a `control/session_refresh` request. The server responds with a new password, which replaces the initial password for all subsequent RC4 encryption. This prevents replay attacks.
- **Module**: A namespace in SpiceAPI grouping related functions (e.g., `buttons`, `coin`, `capture`).
- **Function**: A specific operation within a module (e.g., `buttons/read`, `coin/insert`).

## SpiceAPI Modules and Functions

| Module | Functions | Purpose |
|--------|-----------|---------|
| `analogs` | `read`, `write`, `write_reset` | Read/write analog inputs (turntables, sliders) |
| `buttons` | `read`, `write`, `write_reset` | Read/write digital button states |
| `capture` | `get_screens`, `get_jpg` | List game windows, take JPEG screenshots (base64-encoded) |
| `card` | `insert` | Insert a virtual card (e-amusement pass) |
| `coin` | `get`, `set`, `insert`, `blocker_get` | Manage coin count and coin blocker state |
| `control` | `raise`, `exit`, `restart`, `session_refresh`, `shutdown`, `reboot` | Process control and session management |
| `ddr` | *(game-specific)* | Dance Dance Revolution specific functions |
| `drs` | *(game-specific)* | Dance Rush Stardom specific functions |
| `iidx` | `ticker_get`, `ticker_set`, `ticker_reset`, `tapeled_get` | beatmania IIDX specific (ticker text, tape LEDs) |
| `info` | `avs`, `launcher`, `memory` | System info (AVS version, launcher info, memory usage) |
| `keypads` | `write`, `set`, `get` | Keypad input (PIN entry) |
| `lcd` | `info` | LCD panel information |
| `lights` | `read`, `write`, `write_reset` | Read/write cabinet light states |
| `memory` | `write`, `read`, `signature` | Direct memory read/write and signature scanning |
| `resize` | `image_resize_enable`, `image_resize_set_scene` | Window resize control |
| `touch` | `read`, `write`, `write_reset` | Touch screen input |

## Business Rules

- The CLI must connect to a **running** spice2x instance — it cannot start games itself.
- Session refresh happens automatically on every new connection. The server dictates the new encryption password.
- RC4 encryption is optional — an empty password means plaintext communication.
- Request IDs are monotonically increasing (per-process). Session refresh uses a random ID to avoid collisions with the normal sequence.
- Transient connection errors (TCP failures) are retried up to 3 times with automatic reconnect. API-level errors and ID mismatches are never retried.
- Screenshots come back as base64-encoded JPEG data in the response `data` array: `[timestamp, width, height, base64_string]`. By default they are saved to the `capture/` directory with auto-generated timestamped filenames.

## Out of Scope

- This tool does NOT launch or install games
- This tool does NOT modify spice2x configuration files
- This tool does NOT implement a SpiceAPI server — it is client-only
- This tool does NOT provide a GUI — it is strictly CLI
- WebSocket transport (used by some reference clients) is not planned — TCP only

## Common Mistakes to Avoid

- Don't forget that RC4 is stateful — the cipher state carries across all messages on a connection. You cannot create a new cipher per message.
- Don't use the initial password after session refresh — the server's response contains the new password that must replace it.
- Don't retry on `ProtocolError::Api` or `ProtocolError::IdMismatch` — these are deterministic failures, not transient.
- Don't assume response data shapes are uniform across modules — each module/function has its own response format.
- The null-byte terminator (`\x00`) is part of the framing, not part of the JSON payload. Strip it before parsing.
