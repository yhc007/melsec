# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

A Rust library + three binaries for talking to **Mitsubishi MELSEC PLCs** over the **MC Protocol 3E (Binary)** over TCP, plus an optional Kafka publishing path. There is also a **parallel Python implementation** under `python/` (uses `pymcprotocol`) that exists for quick verification, not as a port target.

The workspace-level `/home/root1/Work/CLAUDE.md` notes that `RS_PLC/` and the sibling `melsec/` are overlapping copies. Treat this directory as the active one (recent commits, daemon work, Kafka integration), but check `git log` in both before assuming.

## Build / run

The crate name is `melsec-plc`. There are three binaries sharing `src/lib.rs`:

| Binary | Source | Purpose |
|---|---|---|
| `melsec-plc` (default) | `src/main.rs` | egui/eframe GUI — needs X11 |
| `melsec-plc-tui` | `src/main_tui.rs` | ratatui/crossterm TUI — no X11 |
| `melsec-plc-daemon` | `src/main_daemon.rs` | Headless PLC→Kafka loop |

```bash
cargo build --release                           # build all three
cargo build --release --bin melsec-plc-tui      # just the TUI
cargo run --release --bin melsec-plc-daemon     # run daemon from source
cargo run --example test_d1000_read             # one-shot read against the dev PLC
```

There is no test suite (`cargo test` runs nothing). For ad-hoc verification use `examples/test_d1000_read.rs` or `examples/test_kafka.rs`, or the Python scripts in `python/` (`./setup.sh` once, then `python plc_reader.py`).

GUI binary on a headless host: `./run_gui.sh` wraps Xvfb + `LIBGL_ALWAYS_SOFTWARE=1`. TUI requires a real tty — it will fail under `TERM=dumb`.

## Daemon deployment

The daemon is configured **entirely through environment variables** (see `DaemonConfig::from_env` in `src/main_daemon.rs`); there is no config file format of its own — `config/daemon.env` is just a `KEY=value` file consumed by systemd `EnvironmentFile=`.

```
PLC_IP, PLC_PORT, KAFKA_BROKERS, KAFKA_TOPIC,
START_ADDRESS, READ_COUNT, READ_INTERVAL_MS, RUST_LOG, PACKET_TRACE
```

systemd install layout (set up by `install-daemon.sh`):

| Path | Purpose |
|---|---|
| `/usr/local/bin/melsec-plc-daemon` | binary |
| `/etc/melsec-plc/daemon.env` | env file (edit this, then `systemctl restart`) |
| `/etc/systemd/system/melsec-plc-daemon.service` | unit (runs as `root1:root1`) |
| `/var/lib/melsec-plc/` | WorkingDirectory |

The unit pins `User=root1`, so it is **not** portable across hosts — change before deploying elsewhere. `manage-daemon.sh`, `install-daemon.sh` and `uninstall-daemon.sh` all take an instance argument (`1`, `2` or `all`, defaulting to `all`); `manage-daemon.sh`'s `config` and `kafka` subcommands require a specific instance. `uninstall-daemon.sh` removes the install but backs up the env file to `/tmp`.

## Default dev target

Throughout the code (defaults in `DaemonConfig::from_env`, the GUI/TUI initial state, examples) the dev PLC is **`192.168.21.112:5010`**, network=0, PC=0xFF, reading **`D1000`–`D1009`**. Both deployed instances override this with `READ_COUNT=17` (`D1000`–`D1016`) in their env files. Recent commit `00f7a4d` moved the default port from 5007 → 5010 — when reading old docs/screenshots assume 5007 may appear.

## Architecture notes

`src/lib.rs` exports the public surface:

- `MelsecClient` (`client.rs`) — async TCP client, owns one `TcpStream`. `connect_str(ip, port, network, pc)` is the usual entry point. Per-call timeout via `set_timeout`. Methods: `read_words`, `read_word`, `read_bits`, `write_word`, `write_words`, `write_bit`, `disconnect`.
- `FrameBuilder` (`protocol.rs`) — builds/parses the MC Protocol 3E binary frames. **Only this file knows the wire format** — header layout, error-code table, byte order. The 11-byte header + little-endian fields are hand-rolled with `bytes::BytesMut`; if you change framing, the response parser at `parse_response` / `parse_bit_response` must change in lockstep (header size constants 11 / 13 / 15 are duplicated there).
- `Device` (`device.rs`) — the `(BitDevice, WordDevice)` enums and `Device::from_str("D100")` parser. Adding a device type means updating both `code()` and `from_str()`.
- `KafkaProducer` (`kafka_producer.rs`) — thin `rdkafka::FutureProducer` wrapper. Ships JSON-serialized `PlcReadResult` (`kafka_types.rs`) keyed by Unix timestamp.
- `MelsecError` (`error.rs`) — `thiserror`-based; the variant `PlcError(code, msg)` carries the MC Protocol error code (see below).

The TUI/daemon build their own `PlcReadResult` directly rather than going through `PlcReadResult::new` — that constructor is unused by the binaries and only useful if you have parallel `(addr, value)` pairs and a separate address-string list.

Both `client.rs` and `main_tui.rs` write hex packet/event traces to `packet_debug.log` and `tui_debug.log` in **CWD** (not stderr, not journald). These files are committed to the repo; if you see large diffs in them, it is just a fresh run, not real changes — don't include them in commits.

## MC Protocol gotcha: error 0xC056

The most common failure when bringing up a new PLC is `PlcError(0xC056, "지정된 디바이스가 범위를 벗어났거나 잘못된 주소입니다")` — the address is outside this CPU model's D-register range. `PROBLEM_SOLUTION.md` documents the diagnosis. Practical rule: if reads time out or return 0xC056, drop to `D0`/`D100` first, then climb. FX series tops out at `D7999`; `D7000+` is not portable.

The full error-code table is duplicated inline in `protocol.rs` at both `parse_response` and `parse_bit_response` — keep them in sync.

## Conventions

- **Korean is the default language** for log messages, comments, error strings, and TUI labels. Keep new code in Korean unless the user asks otherwise (matches the workspace-wide convention).
- The Kafka topic names `melsec-plc-data-1` (PLC 192.168.21.112) and `melsec-plc-data-2` (192.168.21.114) are **this project's own convention** and does NOT match the workspace `cnc-*` topic family used by the FOCAS/MTConnect adapters — this daemon is a standalone collector, not part of the CNC pipeline.
- `Cargo.lock` is in `.gitignore` here (atypical for a binary crate — be aware before committing it).
- Many `.md` files at the repo root are user-facing run/troubleshooting guides in Korean (`RUN.md`, `RUN_TUI.md`, `DAEMON_GUIDE.md`, `QUICKSTART.md`, `TROUBLESHOOTING.md`, `PROBLEM_SOLUTION.md`). Treat them as documentation, not stale scratch.
