# Package Structure

## Directory Layout

```
spice2x-cli/
├── Cargo.toml                → Package manifest (name, version, dependencies)
├── Cargo.lock                → Locked dependency versions (committed — binary project)
├── LICENSE                   → Apache 2.0
├── README.md                 → Project overview, all commands documented, protocol summary
├── .gitignore                → Ignores target/, packet-logs/, .spec/tasks/**/events.csv|state.json
├── src/
│   ├── main.rs               → Entry point: parse CLI, connect, dispatch, format output, exit codes
│   ├── cli.rs                → clap derive structs: Cli (global opts), Commands enum, all subcommands
│   ├── output.rs             → Formatter trait, JsonFormatter (pretty JSON), TextFormatter (human-readable)
│   ├── commands/
│   │   ├── mod.rs            → Command dispatch (routes Commands enum to handler functions)
│   │   ├── info.rs           → info avs/launcher/memory — query system information
│   │   ├── control.rs        → control raise/exit/restart/shutdown/reboot — process lifecycle
│   │   ├── buttons.rs        → buttons read/write/write-reset — digital button states
│   │   ├── analogs.rs        → analogs read/write/write-reset — analog input values
│   │   ├── coin.rs           → coin get/set/insert — coin management
│   │   ├── card.rs           → card insert — virtual card insertion
│   │   ├── keypads.rs        → keypads get/write/set — keypad input control
│   │   ├── capture.rs        → capture get-screens/get-jpg — screenshot capture with base64/file I/O
│   │   └── lights.rs         → lights read — light state observation with optional name filtering
│   └── protocol/
│       ├── mod.rs             → Re-exports: Connection, ProtocolError, and submodules
│       ├── connection.rs      → TCP connection management, RC4 encryption, framing, session refresh, retry
│       ├── rc4.rs             → RC4 stream cipher (KSA + PRGA), stateful encrypt/decrypt
│       ├── request.rs         → Request struct: JSON serialization, monotonic ID generation
│       ├── response.rs        → Response struct: JSON deserialization, ID validation, error extraction
│       └── error.rs           → ProtocolError enum (thiserror): Connection, Api, IdMismatch, Json, etc.
└── .spec/                     → Workspace metadata
    ├── workspace-manifest.json → Package registry, steering index, feature tracking
    ├── system/                → Steering files (this directory)
    └── tasks/                 → Feature task tracking (events, state, requirements, design)
```

## Where Things Go

### Adding a new CLI subcommand
1. Add the subcommand enum variant in `src/cli.rs` (clap derive structs live here)
2. Create a handler file in `src/commands/` (e.g., `src/commands/buttons.rs`)
3. Add a match arm in `src/commands/mod.rs` to dispatch to the new handler
4. Declare the module in `src/commands/mod.rs` with `mod buttons;`

### Adding a new SpiceAPI module wrapper
1. Create a new file in `src/commands/` with a public `execute` function that takes `&mut Connection` and the subcommand enum
2. Call `conn.request(module, function, params)` and return `anyhow::Result<Value>`
3. Wire into the dispatch match in `src/commands/mod.rs`

### Adding protocol-level functionality
1. Modify files in `src/protocol/` — this is the only place that touches TCP, RC4, or JSON framing
2. `Connection` is the public API surface — callers use `Connection::new()` and `Connection::request()`
3. Keep `Request`, `Response`, `Rc4` as internal implementation details of `Connection`

## Code Organization Patterns

- **Protocol layer is self-contained**: Everything in `src/protocol/` deals with the wire protocol. No business logic, no CLI concerns. The public API is `Connection::new()` and `Connection::request()`.
- **Command layer is thin**: Each `src/commands/*.rs` file builds params, calls `conn.request()`, and transforms the response. No protocol knowledge leaks into commands.
- **Flat module structure**: No deep nesting. `src/protocol/mod.rs` re-exports the public types.
- **Tests co-located**: Unit tests live in `#[cfg(test)] mod tests` blocks at the bottom of each file. No separate `tests/` directory for unit tests.

## Naming Conventions

| Thing | Convention | Example |
|-------|-----------|---------|
| Files | `snake_case.rs` | `connection.rs`, `rc4.rs` |
| Structs | `PascalCase` | `Connection`, `Request`, `Response` |
| Enums | `PascalCase` with `PascalCase` variants | `ProtocolError::IdMismatch` |
| Functions/methods | `snake_case` | `session_refresh()`, `send_request()` |
| Constants | `UPPER_SNAKE_CASE` | `TIMEOUT`, `MAX_RETRIES`, `NEXT_ID` |
| Modules | `snake_case` | `mod protocol`, `mod connection` |

## Key Files by Concern

| If you need to... | Look at... |
|-------------------|-----------|
| Add or modify CLI flags/subcommands | `src/cli.rs` (all clap derive structs) |
| Add a new command handler | `src/commands/mod.rs` (dispatch), `src/commands/{module}.rs` (handler) |
| Change output formatting | `src/output.rs` (Formatter trait, JsonFormatter, TextFormatter) |
| Understand the wire protocol | `src/protocol/connection.rs` (framing, encryption, session refresh) |
| See how requests are built | `src/protocol/request.rs` |
| See how responses are parsed | `src/protocol/response.rs` |
| Understand error types and retry decisions | `src/protocol/error.rs`, `connection.rs` retry loop |
| Check RC4 correctness | `src/protocol/rc4.rs` (tested against RFC 6229 vectors) |

## Files You Should Never Edit

- `Cargo.lock` — Auto-generated by Cargo. Only modify indirectly via `Cargo.toml` changes + `cargo update`.
- `.spec/workspace-manifest.json` — Managed by the workspace tooling, not hand-edited.
