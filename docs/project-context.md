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

The 90 % line and function gate applies to the unit-testable pure logic in `src/core/` and `src/platform/` (errors, geometry clamping and persistence, directory resolution, single-instance lock, rolling log writer), which measures 96.51 % lines and 92.92 % functions. `src/ui/window.rs`, the Iced application loop in `src/lib.rs` and the binary edge in `src/main.rs` are excluded from unit coverage because they require a display server; the end-to-end target covers the shell lifecycle instead. The overall project percentage (87.75 % lines, 84.25 % functions) is therefore reported as a trend rather than a hard gate until feature work begins, per design D6. The 90 % threshold itself is never lowered: the display-bound files are removed from the measurement by configuration, and `coverage/lcov.info` carries only `src/core/**` and `src/platform/**` records. `cargo-llvm-cov` 0.9.1 reads no configuration file of its own and accepts the exclusion only through `--ignore-filename-regex`, so the exclusion is injected with `LLVM_COV_FLAGS='-ignore-filename-regex=(^|/)src/((main|lib)\.rs|ui/)'`, which the tool forwards verbatim to `llvm-cov export`. Cargo's `[env]` configuration reaches only build scripts, rustc, `cargo run` and `cargo test` — never an external subcommand — so `.cargo/config.toml` cannot carry the variable and the capture command must export it: `LLVM_COV_FLAGS='-ignore-filename-regex=(^|/)src/((main|lib)\.rs|ui/)' nsoft-specboot collect bootstrap-rust-desktop-shell`. A bare `cargo llvm-cov --lcov` or a bare `collect` invocation measures the whole project and records the trend percentages instead of the gate, which is why the capture command is recorded here and in `coverage-baseline.json` (`measurement_configuration`).

#### SonarQube Quality Gate

The scan reports `SCAN_STATUS: SUCCESS` and `QUALITY_GATE: PASS`. The gate passes because its conditions are evaluated against the configuration held on the SonarQube server, which carries no hard 90 % coverage condition during this bootstrap phase; coverage is tracked as a trend until feature work begins, per design D6. `sonar-project.properties` declares the analysis inputs only (`sonar.sources`, `sonar.tests`, the LCOV report and the Clippy report) and holds no gate threshold, so the recorded gate result shows that the configured conditions were satisfied and is not evidence that coverage met the 90 % target. That target applies to the unit-testable scope above (96.49 % lines, 92.86 % functions); the overall project percentage is reported alongside it as a trend. The reasoning is recorded here and in the `sonarqube_gate_note` field of `coverage-baseline.json`, and is tracked as a gap in `docs/standards-gaps.md`.

#### Evidence Capture

`nsoft-specboot collect` writes `openspec/changes/<change>/logs/evidence.json` (`schemaVersion: 3`) with the app type, stack and mode plus one record per mandatory producer: `unit-tests` (`cargo test`), `coverage` (`cargo llvm-cov --lcov --output-path coverage/lcov.info`) and `e2e` (`cargo test --test e2e`). Each record carries its command, exit code and status, and the coverage record additionally carries the metrics and thresholds of that run. The manifest establishes that the producers ran and passed; it does not by itself establish that the implementation meets the specifications. Implementation correctness is cross-checked through the requirement-to-code mapping of the change specs and through the end-to-end suite in `tests/e2e.rs`, whose 5 tests cover isolated user data with cleanup, window title presence, single-instance lock rejection, geometry restore and clamp, and logging with the panic hook. The coverage metrics inside the manifest belong to the capture run it records; the current baseline is the one in `coverage-baseline.json`. Because the capture exports the coverage scope above, the coverage record carries the unit-testable metrics (96.51 % lines, 92.92 % functions, 100 % branches on a branch-free crate) against the 90 % line and function thresholds, so no recorded metric contradicts the threshold it is measured against; `statements` is `null` because the LCOV format carries no statement counter.
