# Tasks: 20260319-initial-cli-core

## Task Breakdown

- [x] Task 1: Project scaffolding and protocol layer
- [x] Task 2: CLI structure, output formatting, and main dispatch
- [x] Task 3: Info and Control commands
- [x] Task 4: Buttons and Analogs commands
- [x] Task 5: Coin, Card, and Keypads commands
- [x] Task 6: Capture and Lights commands

---

### Task 1: Project scaffolding and protocol layer

**Goal**: Cargo project initialized with all dependencies, and the full protocol layer compiles and is unit-testable.

**Scope**:
- `Cargo.toml` with all dependencies (clap, serde, serde_json, anyhow, thiserror, base64, chrono)
- `src/protocol/rc4.rs` — RC4 cipher (KSA + PRGA), verified against reference implementations
- `src/protocol/error.rs` — `ProtocolError` enum via thiserror
- `src/protocol/request.rs` — Request struct + JSON serialization with monotonic ID
- `src/protocol/response.rs` — Response parsing, ID validation, error extraction
- `src/protocol/connection.rs` — TCP connect, RC4 encrypt/decrypt, null-byte framing, session refresh, retry logic (3 attempts)
- `src/protocol/mod.rs` — re-exports
- Minimal `src/main.rs` that compiles

**Tests**: Unit tests for RC4 (known test vectors vs reference), request serialization, response parsing, error types

**Dependencies**: None — first task

---

### Task 2: CLI structure, output formatting, and main dispatch

**Goal**: The CLI parses all global flags and subcommands, dispatches to command handlers, and formats output as JSON or text.

**Scope**:
- `src/cli.rs` — clap derive structs: `Cli` (global opts), `Commands` enum, all subcommand enums
- `src/output.rs` — `Formatter` trait, `JsonFormatter`, `TextFormatter`
- `src/main.rs` — parse args, establish connection, dispatch to command module, format output, handle errors with exit codes
- `src/commands/mod.rs` — command dispatch function (stub implementations that return placeholder data)

**Tests**: CLI parsing tests (valid args, missing args, defaults), formatter output tests

**Dependencies**: Task 1 (protocol layer)

---

### Task 3: Info and Control commands

**Goal**: First real end-to-end commands work against a running spice2x instance.

**Scope**:
- `src/commands/info.rs` — `info avs`, `info launcher`, `info memory`
- `src/commands/control.rs` — `control raise`, `control exit`, `control restart`, `control shutdown`, `control reboot`
- Wire into CLI dispatch (replace stubs)

**Tests**: Unit tests for request construction and response handling per command

**Dependencies**: Task 2 (CLI + dispatch)

---

### Task 4: Buttons and Analogs commands

**Goal**: Core game interaction commands for reading and writing button/analog states.

**Scope**:
- `src/commands/buttons.rs` — `buttons read`, `buttons write`, `buttons write-reset`
- `src/commands/analogs.rs` — `analogs read`, `analogs write`, `analogs write-reset`
- Wire into CLI dispatch

**Tests**: Unit tests for request construction, param formatting, response handling

**Dependencies**: Task 2 (CLI + dispatch)

---

### Task 5: Coin, Card, and Keypads commands

**Goal**: Game session management commands — insert coins, authenticate with cards, enter PINs.

**Scope**:
- `src/commands/coin.rs` — `coin get`, `coin set`, `coin insert`
- `src/commands/card.rs` — `card insert`
- `src/commands/keypads.rs` — `keypads get`, `keypads write`, `keypads set`
- Wire into CLI dispatch

**Tests**: Unit tests for request construction, param formatting, response handling

**Dependencies**: Task 2 (CLI + dispatch)

---

### Task 6: Capture and Lights commands

**Goal**: Screenshot capture with file I/O and base64 handling, plus lights observability.

**Scope**:
- `src/commands/capture.rs` — `capture get-screens`, `capture get-jpg` with `--screen`, `--quality`, `--divide`, `--output`, `--base64` flags
- Base64 decode → JPEG file write, auto-generated filename with timestamp
- File path printed to stderr
- `src/commands/lights.rs` — `lights read` with optional name filtering
- Wire into CLI dispatch

**Tests**: Unit tests for capture param construction, filename generation, base64 decode path, lights filtering

**Dependencies**: Task 2 (CLI + dispatch)
