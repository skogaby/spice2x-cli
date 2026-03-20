# Technical Context

## Technology Stack

| Component | Version | Purpose |
|-----------|---------|---------|
| Rust | 2021 edition | Language |
| Cargo | standard | Build system |
| `clap` 4.x (derive) | CLI framework | Argument parsing with derive macros |
| `serde` + `serde_json` 1.x | Serialization | JSON request/response encoding |
| `anyhow` 1.x | Error handling | Application-level errors with context |
| `thiserror` 2.x | Error handling | Typed protocol errors with Display/Error derives |
| `base64` 0.22 | Encoding | Decoding screenshot JPEG data from SpiceAPI |
| `chrono` 0.4 | Time | Timestamps for auto-generated screenshot filenames |

## Build & Run

```bash
cargo build                    # Debug build
cargo build --release          # Release build
cargo test                     # Run all tests
cargo run -- --help            # Run with args (once CLI dispatch is implemented)
```

No external services or environment variables are required to build. The binary is self-contained.

## Architecture Decisions

### Why Rust (not Python)?
The Python reference client exists but this CLI is meant for AI agent automation where:
- A single compiled binary is easier to distribute and invoke than a Python environment
- Strong typing catches protocol mismatches at compile time
- Performance matters less than correctness, but Rust gives both

### Why synchronous TCP (not async)?
SpiceAPI is strictly request-response with one outstanding request at a time. The protocol has no multiplexing or streaming. Async would add complexity (tokio dependency, async infection) with no benefit for a sequential CLI tool.

### Why `thiserror` for protocol errors, `anyhow` for application errors?
- `thiserror` in `protocol/error.rs`: Protocol errors are a closed set (`Connection`, `Api`, `IdMismatch`, `Json`, `MalformedResponse`, `RetriesExhausted`). Callers need to match on variants to decide retry behavior.
- `anyhow` at the application level (main, CLI dispatch): Errors bubble up to the user as messages. No need to match on variants — just display them.

### Why custom RC4 instead of a crate?
RC4 is 30 lines of code. The spice2x server uses a textbook RC4 implementation. A crate would add a dependency for something trivially verifiable against the reference implementations. The implementation is tested against RFC 6229 vectors.

### Why monotonic IDs with AtomicU64?
The Python reference uses a global lock-protected counter. AtomicU64 with Relaxed ordering is the idiomatic Rust equivalent — lock-free, correct for a single-process monotonic counter. Session refresh uses a random ID (via `RandomState` hasher) to avoid colliding with the normal sequence.

## Protocol Wire Format

```
Client → Server:  JSON_BYTES + 0x00    (optionally RC4-encrypted, including the 0x00)
Server → Client:  JSON_BYTES + 0x00    (optionally RC4-encrypted, including the 0x00)
```

Request JSON: `{"id":1,"module":"buttons","function":"read","params":[]}`
Response JSON: `{"id":1,"errors":[],"data":[{"name":"BT_A","value":0.0}, ...]}`

The RC4 cipher is initialized once per connection (or per session refresh) and its state is continuous across all messages — it is NOT reset per message.

## Retry and Reconnect Strategy

- Max 3 attempts per `Connection::request()` call
- On transient errors (TCP I/O failures): reconnect (new TCP socket + session refresh), then retry
- On deterministic errors (`Api`, `IdMismatch`): fail immediately, no retry
- On all retries exhausted: return `RetriesExhausted` wrapping the last error

## Key Integration Point

| System | Protocol | Purpose | Timeout |
|--------|----------|---------|---------|
| spice2x instance | TCP + JSON + RC4 | Game control and observation | 3s read/write |

## Local Development Gotchas

- You need a running spice2x instance to test against. There is no mock server in this repo.
- RC4 cipher state is cumulative. If you add debug logging that reads/writes extra bytes through the cipher, you'll desync the stream and corrupt all subsequent messages.
- `Cargo.lock` is committed (binary project, not library). Don't `.gitignore` it.
