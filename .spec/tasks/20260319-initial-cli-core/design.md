# Design: 20260319-initial-cli-core

**Requirements**: [requirements.md](requirements.md)

---

## Overview

A Rust CLI binary that communicates with a running spice2x instance over the SpiceAPI TCP protocol. The CLI accepts global connection flags, then dispatches to subcommands that map 1:1 to SpiceAPI modules/functions. Output is either JSON (for agent consumption) or human-readable text.

---

## Architecture Decisions

### Decision 1: Single Binary Crate (not workspace/library)

**Problem**: Should the SpiceAPI protocol layer be a separate library crate?

**Decision**: Single binary crate with internal modules. No library crate.

**Rationale**: This tool is purpose-built for CLI use. There are no other consumers of the protocol layer. Extracting a library adds build complexity and crate boundaries for zero benefit today. If a library is needed later (e.g., for a GUI or another tool), the `protocol` module is already cleanly separated and can be extracted with minimal refactoring.

**Alternative**: Cargo workspace with `spiceapi` lib + `spice2x-cli` bin. Rejected — premature abstraction for a single consumer.

### Decision 2: Synchronous TCP (no async runtime)

**Problem**: Should we use tokio/async-std for networking?

**Decision**: Use `std::net::TcpStream` with blocking I/O.

**Rationale**: SpiceAPI is strictly request-response — one request in flight at a time, one connection per CLI invocation. There is no concurrency, no multiplexing, no streaming. An async runtime adds ~3MB to binary size, increases compile time, and adds complexity for zero throughput benefit. The CLI sends one request and exits.

**Alternative**: tokio with `TcpStream`. Rejected — async overhead with no concurrency to exploit.

**Tradeoff**: If future features need concurrent requests (e.g., polling lights while sending inputs), this decision would need revisiting. That's out of scope per requirements.

### Decision 3: clap (derive) for CLI Parsing

**Problem**: How to parse the `spice2x-cli --host H --port P --password S -- buttons read` invocation pattern.

**Decision**: Use `clap` with derive macros. Global connection flags on the root struct, subcommands for each SpiceAPI module, nested subcommands for functions.

**Rationale**: clap is the de facto Rust CLI framework. Derive macros give compile-time validation of the CLI structure. The `--` separator in the requirements naturally maps to clap's subcommand parsing — global flags before the subcommand, subcommand args after.

**Note on `--` separator**: clap doesn't require `--` between global flags and subcommands. The standard Unix CLI convention applies — global flags before the subcommand, subcommand args after: `spice2x-cli --host 192.168.1.10 buttons read`. No separator needed. clap disambiguates subcommand names from flag values at parse time.

### Decision 4: Inline RC4 Implementation

**Problem**: RC4 cipher needed for SpiceAPI encryption. Use a crate or implement?

**Decision**: Implement RC4 inline (~30 lines of Rust).

**Rationale**: RC4 is a trivial algorithm (KSA + PRGA). The reference implementations (Python, C++) all implement it inline. Adding a crate dependency for 30 lines of code increases supply chain surface for no benefit. The implementation is directly verifiable against the reference Python/C++ code.

**Alternative**: `rc4` crate from crates.io. Rejected — unnecessary dependency for a trivial algorithm.

### Decision 5: Error Strategy — `anyhow` for Application, Typed Errors for Protocol

**Problem**: How to handle errors across the CLI.

**Decision**: Use `anyhow::Result` at the CLI/command layer for ergonomic error propagation. Define a small `ProtocolError` enum (via `thiserror`) for the protocol layer to distinguish connection failures, timeout, malformed responses, and API-level errors.

**Rationale**: The protocol layer needs typed errors so the retry logic can distinguish retryable errors (connection reset, timeout) from non-retryable ones (API error response, malformed JSON). The CLI layer just needs to display errors and set exit codes — `anyhow` is ideal there.

### Decision 6: Output Formatting as a Trait

**Problem**: Commands need to produce either JSON or human-readable text output.

**Decision**: Each command returns a structured result type. A `Formatter` trait with `JsonFormatter` and `TextFormatter` implementations handles rendering. JSON mode serializes the raw API response data. Text mode formats it for human readability.

**Rationale**: Keeps command logic decoupled from presentation. Commands don't know or care about the output format. Adding new formats (e.g., YAML, table) later requires only a new `Formatter` implementation.

---

## Component Design

### Module Layout

```
src/
├── main.rs              → Entry point: parse CLI, connect, dispatch, format output
├── cli.rs               → clap derive structs (Cli, GlobalOpts, Commands, subcommands)
├── protocol/
│   ├── mod.rs           → Re-exports
│   ├── connection.rs    → TCP connection, RC4 encryption, send/receive with null-byte framing
│   ├── rc4.rs           → RC4 cipher (KSA + PRGA)
│   ├── request.rs       → Request struct + JSON serialization
│   ├── response.rs      → Response struct + JSON deserialization
│   └── error.rs         → ProtocolError enum (thiserror)
├── commands/
│   ├── mod.rs           → Command dispatch
│   ├── buttons.rs       → buttons read/write/write-reset
│   ├── analogs.rs       → analogs read/write/write-reset
│   ├── coin.rs          → coin get/set/insert
│   ├── capture.rs       → capture get-screens/get-jpg (base64 decode + file write)
│   ├── info.rs          → info avs/launcher/memory
│   ├── card.rs          → card insert
│   ├── keypads.rs       → keypads get/write/set
│   ├── lights.rs        → lights read
│   └── control.rs       → control raise/exit/restart/shutdown/reboot
└── output.rs            → Formatter trait, JsonFormatter, TextFormatter
```

### Component Responsibilities

| Component | Responsibility |
|-----------|---------------|
| `main.rs` | Parse CLI args, establish connection (with session refresh), dispatch to command, format and print output, set exit code |
| `cli.rs` | Defines the full CLI structure via clap derive. No logic — pure data definition |
| `protocol::connection` | Manages TCP socket lifecycle. Handles RC4 encrypt/decrypt on send/receive. Implements null-byte message framing. Performs session refresh on connect. Retry logic (3 attempts) on transient failures |
| `protocol::rc4` | Stateful RC4 stream cipher. Consumes key bytes, produces keystream for XOR |
| `protocol::request` | Builds JSON request payloads: `{"id", "module", "function", "params"}`. Manages monotonic request ID counter |
| `protocol::response` | Parses JSON response: validates `id` match, extracts `errors` and `data` arrays. Returns `ProtocolError::ApiError` if errors array is non-empty |
| `commands/*` | Each module builds a `Request`, sends it via `Connection`, and returns the response data as a `serde_json::Value`. Capture module additionally handles base64 decoding and file I/O |
| `output` | Formats command results for stdout. JSON mode: serialize `serde_json::Value`. Text mode: human-friendly key-value rendering |

### Data Flow

```
CLI args → clap parse → GlobalOpts + Command
                              ↓
                    Connection::new(host, port, password)
                         ↓ (TCP connect)
                         ↓ (RC4 init with password)
                         ↓ (session_refresh → server returns new password → re-init RC4)
                              ↓
                    Command::execute(&connection)
                         ↓ (build Request JSON)
                         ↓ (encrypt + send + null byte)
                         ↓ (receive until null byte + decrypt)
                         ↓ (parse Response JSON, validate ID)
                              ↓
                    Formatter::format(response_data) → stdout
```

### Connection Lifecycle (per CLI invocation)

1. TCP connect to `host:port`
2. If password is non-empty: initialize RC4 cipher with password bytes
3. Send `control/session_refresh` request (with random ID, matching Python reference)
4. Server responds with new password in `data[0]`
5. Re-initialize RC4 cipher with new password
6. Execute the user's command
7. Close TCP connection

### Retry Behavior

Retries happen at the `Connection::request()` level:
- On send/receive failure (connection reset, timeout): reconnect and retry, up to 3 total attempts
- On API-level error (non-empty `errors` array): do NOT retry — propagate immediately
- On response ID mismatch: do NOT retry — propagate as protocol error
- Backoff: none needed (single-shot CLI, not a long-running service)

### Screenshot (Capture) Flow

`capture get-jpg` is the most complex command due to file I/O:

1. Send `capture/get_jpg` with params `[screen, quality, divide]`
2. Response `data` contains `[timestamp, width, height, base64_string]`
3. If `--base64` flag: write base64 string to stdout, done
4. If file output: base64-decode → raw JPEG bytes → write to file
5. File path: `--output <path>` if specified, else auto-generate `capture_YYYYMMDD_HHMMSS.jpg`
6. Print file path to stderr (keeps stdout clean for piping)

---

## Dependencies

| Crate | Purpose | Why This One |
|-------|---------|-------------|
| `clap` (derive) | CLI argument parsing | De facto standard, compile-time validation |
| `serde` + `serde_json` | JSON serialization/deserialization | Universal Rust JSON handling |
| `anyhow` | Application-level error handling | Ergonomic error propagation for CLI |
| `thiserror` | Typed protocol errors | Derive `Error` for `ProtocolError` enum |
| `base64` | Base64 decode for screenshots | Standard, minimal |
| `chrono` | Timestamp for auto-generated filenames | Only used in capture command |

No async runtime. No HTTP client. No TLS. Minimal dependency tree.

---

## Risks and Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Large screenshot responses (multi-MB base64) cause slow receive | Medium | Medium | Pre-allocate receive buffer. The protocol's null-byte termination means we must read the full response regardless — this is inherent to the protocol, not a design flaw. Document that `--divide` reduces image size |
| RC4 cipher implementation bug causes protocol mismatch | High | Low | Verify against Python and C++ reference implementations with known test vectors. RC4 is deterministic — same key + plaintext must produce identical ciphertext across all implementations |
| spice2x server closes connection unexpectedly during command | Medium | Medium | Retry logic (3 attempts) with reconnect handles transient failures. `control` commands (exit, shutdown, reboot) expect connection closure — suppress error for those |
| Future SpiceAPI protocol changes break the client | Low | Low | Protocol is stable (reference implementations haven't changed). JSON-based protocol is forward-compatible — unknown fields in responses are ignored by serde |

---

## Open Questions

1. **Button write params format**: The Python reference passes a list of state objects to `buttons_write`. Need to verify the exact param format expected by the server (is it `[{"name": "btn", "value": 1.0}]` or positional `["btn", 1.0]`?). Will resolve during implementation by reading the server-side `buttons.cpp` handler.

---

## What's NOT in This Design (Intentionally)

- Game-specific modules (iidx, drs, ddr, lcd) — deferred per requirements
- Memory read/write — deferred per requirements
- WebSocket transport — deferred per requirements
- Config file / env var connection settings — deferred per requirements
- Interactive/REPL mode — deferred per requirements
