---
status: customized
source_substrate: scaffold
---
## Project Context

This document records the descriptive, observed state of your project: its current architecture, technology stack, domain conventions, and known deviations from the shipped standards. It is distinct from the normative `docs/standards-*.md` files, which describe how the project SHOULD be structured. Where the two disagree, the normative standard prevails.

> **Important**: Passing `--force` during `init` will overwrite this file, discarding any hand-filled content. If you rely on this file, preserve a backup before running `init --force`.

### Architecture

Desktop application ported to a Rust/Iced stack. The privileged context owns operating-system capabilities: filesystem access, network sockets to the Gitea API, and credential storage through the operating system keystore. The unprivileged UI context renders the interface through the Iced application loop and reaches privileged capabilities only through the declared surface. Window lifecycle (creation, focus, geometry, shutdown) is owned by the privileged context per the Desktop Standards process and trust boundary model.

### Stack

Rust desktop application with the toolchain pinned via `rust-toolchain.toml` (stable channel, edition 2024) and `Cargo.lock` committed. Single crate with layout `src/{main,lib,ui,core,platform}`: `main.rs` is the binary edge (argument handling, logging init, `anyhow` error-to-exit conversion), `lib.rs` exposes the public API and carries `#![forbid(unsafe_code)]`, `ui/` holds the Iced 0.14.x view layer, `core/` holds pure application logic (typed `thiserror` errors, geometry persistence), and `platform/` holds the privileged edge (user-data/cache dirs, single-instance lock file, rolling log writer with panic hook). Dependencies: `anyhow`, `dirs`, `iced 0.14`, `serde`/`serde_json`, `thiserror`. Gitea instance at git.nsoft.com.ar (Gitea 28.3.0, OpenAPI 3.0.3). API requests authenticate with a PAT in the `Authorization: token` header.

### Domain Conventions

Issues are identified unequivocally as `owner/repo#N`, satisfying the forge model of one issue per repository. Time entries are associated with issue comments and the sum of time linked to a comment can be queried. API requests authenticate with a PAT in the `Authorization: token` header and are rejected without a valid PAT.

### Known Deviations from Shipped Standards

None intentional at this stage. Tracked in `docs/standards-gaps.md`: Iced core does not expose role, state and descriptive label to screen readers (accessibility gap requiring AccessKit or platform-specific code), and Iced does not produce installable packages or sign binaries (packaging gap requiring external tools such as `cargo-bundle`).

### Verification and Quality Gates

CI pipeline runs on Linux and Windows: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets`, coverage via `cargo llvm-cov --lcov` with a 90 % line/function baseline (treated as a trend until feature work begins), `cargo audit` and `cargo deny check`. `tests/e2e.rs` runs against an isolated user-data directory and cleans up after execution. SonarQube scan produces a quality-gate result. Branch `main` is protected requiring PR and status checks; Dependabot opens updates for Cargo and GitHub Actions; `Swatinem/rust-cache` provides build caching with cancel-in-progress concurrency.

#### Coverage Measurement Scope

The 90 % line and function gate applies to the unit-testable pure logic in `src/core/` and `src/platform/` (errors, geometry clamping and persistence, directory resolution, single-instance lock, rolling log writer), which measures 96.49 % lines and 92.86 % functions. `src/ui/window.rs`, the Iced application loop in `src/lib.rs` and the binary edge in `src/main.rs` are excluded from unit coverage because they require a display server; the end-to-end target covers the shell lifecycle instead. The overall project percentage (87.71 % lines, 84.13 % functions) is therefore reported as a trend rather than a hard gate until feature work begins, per design D6.
