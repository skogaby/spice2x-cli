# Requirements: 20260319-initial-cli-core

## Goals
Build the foundational Rust CLI that connects to a running spice2x instance via SpiceAPI over TCP, enabling AI agents (and human users) to send inputs, observe game state, and capture screenshots. This initial release covers the core modules needed for autonomous game interaction.

## User Stories

### US-1: Connection Management
As a user, I want to connect to a spice2x instance by specifying host, port, and password via CLI flags, so I can target any running instance.

**Acceptance Criteria:**
- CLI accepts `--host` (default: `localhost`), `--port` (default: `1337`), `--password` (default: empty string) flags
- Global flags appear before the subcommand: `spice2x-cli --host 192.168.1.10 --port 1337 --password secret buttons read`
- Connection performs RC4-encrypted TCP handshake with session refresh per SpiceAPI protocol
- Connection failures produce a clear error message on stderr and exit with non-zero code
- Failed requests retry up to 3 times before reporting failure

### US-2: Output Formatting
As a user, I want to choose between JSON and human-readable output, so the tool works for both scripted and interactive use.

**Acceptance Criteria:**
- CLI accepts `--format` flag with values `json` (default) and `text`
- JSON mode outputs the raw API response data as JSON to stdout
- Text mode outputs a human-friendly representation to stdout
- Errors always go to stderr regardless of format

### US-3: Button Control
As an AI agent, I want to read button states and send button presses, so I can interact with game menus and gameplay.

**Acceptance Criteria:**
- `buttons read` returns all button names and their current states
- `buttons write <name> <state>` sets a button's state (0.0 or 1.0)
- `buttons write-reset [name...]` resets specified buttons (or all if none specified)

### US-4: Analog Control
As an AI agent, I want to read and write analog inputs, so I can control turntables, sliders, and other analog devices.

**Acceptance Criteria:**
- `analogs read` returns all analog names and their current values
- `analogs write <name> <value>` sets an analog's value (float)
- `analogs write-reset [name...]` resets specified analogs (or all if none specified)

### US-5: Coin Management
As an AI agent, I want to insert coins and check the coin count, so I can start game sessions autonomously.

**Acceptance Criteria:**
- `coin get` returns the current coin count
- `coin set <amount>` sets the coin count to a specific value
- `coin insert [amount]` inserts coins (default: 1)

### US-6: Screenshot Capture
As a user, I want to capture screenshots of the running game, so I can visually observe game state.

**Acceptance Criteria:**
- `capture get-screens` lists available screen indices
- `capture get-jpg [options]` captures a screenshot with optional `--screen` (default: 0), `--quality` (default: 70), `--divide` (default: 1)
- Default behavior saves JPEG to a file (auto-generated name with timestamp, e.g., `capture_20260319_163700.jpg`)
- `--output <path>` saves to a specific file path
- `--base64` outputs the base64-encoded JPEG to stdout instead of saving to file
- File save path is printed to stderr so stdout stays clean for piping

### US-7: System Info
As a user, I want to query system information from the running spice2x instance, so I can verify connectivity and inspect the environment.

**Acceptance Criteria:**
- `info avs` returns AVS system info
- `info launcher` returns launcher info
- `info memory` returns memory usage info

### US-8: Card Insert
As an AI agent, I want to insert a virtual card, so I can log into games that require card authentication.

**Acceptance Criteria:**
- `card insert <unit> <card_id>` inserts a card on the specified unit with the given card ID string

### US-9: Keypad Control
As an AI agent, I want to interact with keypads, so I can enter PINs after inserting a card to complete game authentication.

**Acceptance Criteria:**
- `keypads get <keypad>` returns the current keypad state for the specified keypad index
- `keypads write <keypad> <input>` writes input string to the specified keypad
- `keypads set <keypad> <values...>` sets individual key values on the specified keypad

### US-10: Lights Read
As an AI agent, I want to read light states, so I can observe game feedback signals (e.g., which buttons are lit).

**Acceptance Criteria:**
- `lights read [name...]` returns light states (all if no names specified, or filtered by name)

### US-11: Process Control
As a user, I want to control the spice2x process lifecycle, so I can restart or shut down the game remotely.

**Acceptance Criteria:**
- `control raise <signal>` raises a signal
- `control exit [code]` exits the process with optional exit code
- `control restart` restarts the process
- `control shutdown` shuts down the machine
- `control reboot` reboots the machine

## Out of Scope
- Game-specific modules: `iidx`, `drs`, `ddr`, `lcd` (deferred to future feature)
- Memory read/write/signature module (deferred)
- Touch module (deferred)
- Resize module (deferred)
- Lights write/write-reset (deferred — only read included for observability)
- Config file or environment variable based connection settings (future enhancement)
- WebSocket transport (only TCP for now)
- Interactive/REPL mode

## Assumptions
- spice2x is already running and has SpiceAPI enabled on a known host/port
- The SpiceAPI TCP protocol uses JSON messages terminated by null byte (`\x00`), with optional RC4 encryption
- Session refresh happens automatically on connect (server provides new cipher password)
- Default port is 1337 based on common spice2x configuration

## Open Questions
- None at this time — all clarifications resolved.
