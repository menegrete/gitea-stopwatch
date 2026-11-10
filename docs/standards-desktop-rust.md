---
description: Rust desktop stack standards — toolchain and edition policy, workspace and crate layout, error-handling boundaries, the unsafe and FFI policy, framework mapping, the async runtime, logging and tracing, supply-chain policy, formatting and linting, the testing layers with the required end-to-end target and its per-framework driver, the 90 % coverage gate and the SonarQube posture
globs: ["src/**/*.rs", "tests/**/*.rs", "benches/**/*.rs", "examples/**/*.rs", "Cargo.toml", "clippy.toml"]
alwaysApply: true
---

# Rust Desktop Standards and Best Practices

## Table of Contents

- [Overview](#overview)
- [Toolchain and Edition Policy](#toolchain-and-edition-policy)
- [Crate and Workspace Layout](#crate-and-workspace-layout)
- [Error Handling Boundaries](#error-handling-boundaries)
- [Unsafe and FFI Policy](#unsafe-and-ffi-policy)
- [Framework Mapping](#framework-mapping)
- [Async Runtime Policy](#async-runtime-policy)
- [Logging and Tracing](#logging-and-tracing)
- [Dependency and Supply-Chain Policy](#dependency-and-supply-chain-policy)
- [Formatting and Linting](#formatting-and-linting)
- [Testing Layers](#testing-layers)
- [End-to-End Testing and the Driver Matrix](#end-to-end-testing-and-the-driver-matrix)
- [Coverage and the 90 % Gate](#coverage-and-the-90-gate)
- [SonarQube Posture](#sonarqube-posture)
- [Development Workflow](#development-workflow)

---

## Overview

This document defines the normative standard for the **Rust stack** of the desktop Application Type: how the toolchain is pinned, how the crate is laid out, where errors and `unsafe` are allowed, how the desktop framework is mapped onto the process and trust boundary, how the code is traced, supplied, formatted, tested and analyzed. It refines the [Desktop Standards](./standards-desktop.md) with the rules that follow from Rust and SHALL NOT restate or relax them; the process and trust boundary model, the local state rules, the accessibility expectations and the performance budgets stated there remain in force.

Read this document together with the [Core Standards](./standards-core.md), the [Desktop Standards](./standards-desktop.md) and the [Documentation Standards](./standards-documentation.md); the repository workflow and its mandatory steps are described in the [Development Guide](./development-guide.md).

## Toolchain and Edition Policy

- The toolchain SHALL be pinned by a committed `rust-toolchain.toml` naming one exact channel, so every developer and every CI leg compiles with the same compiler. A floating `stable` channel MUST NOT be used for a release build.
- The edition SHALL be declared once in `[package] edition` of `Cargo.toml` and SHALL be the current edition the pinned toolchain supports. An edition upgrade SHALL be its own change, with the migration record committed.
- Formatting and lint components SHALL be part of the pinned toolchain: `rustfmt` and `clippy` SHALL be declared in `rust-toolchain.toml` so a machine without them fails loudly instead of silently skipping a gate.
- A subcommand the project's workflow depends on — coverage, auditing, denials, packaging — SHALL be installed through `cargo install --locked` from a pinned version and SHALL be documented in the development guide. A subcommand that is missing SHALL be reported as a missing prerequisite, never worked around with an interactive install during a verification run.
- The release profile SHALL be declared explicitly: optimisation level, LTO, `codegen-units`, panic strategy and debug-symbol handling are build inputs, so they belong in the committed manifest rather than in a build script's environment.
- A `rust-version` (MSRV) SHALL be declared when the crate is consumed as a library; a desktop binary SHALL declare the pinned toolchain as its only supported compiler.

## Crate and Workspace Layout

```text
desktop-app/
├── Cargo.toml            # [package] or [workspace]; one declared source of dependencies
├── Cargo.lock            # committed for a binary
├── src/
│   ├── main.rs           # binary edge: argument parsing, runtime setup, error reporting
│   ├── lib.rs            # library root: the modules the tests address
│   ├── ui/               # interface layer: views, state, event wiring
│   ├── core/             # application logic: no I/O, no framework type in its signatures
│   └── platform/         # the privileged edge: filesystem, keyring, child processes, native APIs
├── tests/                # integration tests, one binary per file; tests/e2e.rs is required
├── benches/              # benchmarks behind the criterion harness
└── examples/             # runnable examples, compiled by `cargo test --all-targets`
```

- A single-crate desktop application SHALL use this layout. A workspace SHALL be introduced only when a second deliverable crate genuinely exists (a plugin, a shared library, a second binary), and then `[workspace.dependencies]` SHALL be the single place a dependency version is written.
- `src/lib.rs` SHALL hold the logic and `src/main.rs` SHALL hold only the binary responsibilities: reading arguments, building the runtime, wiring the framework entry point, and turning the top-level error into an exit code and a message. Logic in `main.rs` SHALL NOT be reachable only from `main.rs`.
- Pure logic SHALL live in `src/core/` and SHALL depend on nothing that opens a file, a socket, a process or a framework window. A capability it needs SHALL be a generic parameter or a trait object passed in by the caller, so the logic is testable without a running application.
- Every capability that leaves the process — filesystem, credential store, network socket, child process, notification, clipboard — SHALL live under `src/platform/` and SHALL be reachable from the interface only through the declared surface the [Desktop Standards](./standards-desktop.md) require.
- `Cargo.lock` SHALL be committed for a binary application, and a dependency bump SHALL land with the lockfile change in the same commit.
- Module visibility SHALL be the minimum that compiles: `pub` on a module that no external consumer addresses is a defect, and a `pub` item in a binary crate SHALL be justified by a test that uses it.
- A `build.rs` script SHALL be reserved for what a build genuinely needs (a generated code table, a linkage probe). It MUST NOT compile application logic, and it SHALL be deterministic given the same inputs.

## Error Handling Boundaries

- Library-shaped code — `src/lib.rs`, `src/core/`, `src/platform/` — SHALL define its own error type with `thiserror`, one variant per failure mode the caller can act on. A `Box<dyn Error>`, a bare `String`, or an opaque `anyhow::Error` MUST NOT appear in a function signature that a test or a sibling module calls.
- The binary edge — `src/main.rs` and the framework's entry point — SHALL use `anyhow::Result` to add context as the error travels out. `anyhow` MUST NOT be a dependency of the library modules: the boundary is exactly where the concrete error is erased.
- A fallible function SHALL return `Result`. `unwrap`, `expect` and panicking indexing SHALL NOT appear on a path a user action can reach; where a value is genuinely infallible by construction, the reasoning SHALL be a comment on the same line.
- Every error variant SHALL carry the identifiers a reader needs to diagnose it (the path, the query, the operation) and SHALL NOT carry a raw pointer, a full file body, or a credential.
- An error SHALL be converted to a user-visible message at exactly one place — the edge that renders it — and the message SHALL NOT be the `Debug` rendering of the error tree.
- A panicking path SHALL be a defect, not a strategy: recovery from a panic is not an error-handling mechanism, and a `catch_unwind` SHALL be justified in a comment next to a boundary that genuinely needs it (an FFI callback, a plugin host).
- An error returned across the trust boundary SHALL be a typed, serializable value, as the [Desktop Standards](./standards-desktop.md) require; a Rust error type SHALL be mapped explicitly into that shape, never serialized by deriving `Serialize` on the internal type.

## Unsafe and FFI Policy

- `unsafe` SHALL be forbidden by default. The crate SHALL carry `#![forbid(unsafe_code)]` at its root, and a module that genuinely needs an exception SHALL opt in with a documented `#[allow(unsafe_code)]` at the narrowest possible scope.
- Every `unsafe` block SHALL be preceded by a `// SAFETY:` comment stating the invariant the block relies on and why the surrounding code establishes it. A block without that comment SHALL be treated as a review-blocking defect.
- An exception SHALL be justified against the alternative: a safe crate that already performs the operation, a `bindgen`-generated binding with its own documentation, or a safe wrapper crate. "It is faster" SHALL NOT be the justification; a measured budget regression SHALL be.
- FFI SHALL cross exactly one boundary module under `src/platform/`. The rest of the crate SHALL call a safe Rust function that encapsulates the call, converts the result, and owns the memory.
- An FFI call SHALL validate every pointer, length and handle it receives before dereferencing it, and SHALL NOT assume a foreign caller passed what the signature claims.
- `unsafe` code SHALL be covered by a test that exercises the documented invariant, including the failure path: a test that only proves the happy path does not prove the safety argument.
- The crate SHALL run its test suite under `cargo miri test` where the toolchain supports it, and a Miri finding SHALL be treated as a blocker rather than a warning.
- The desktop trust boundary SHALL NOT be widened by FFI: a native capability reached through FFI SHALL be exposed through the same declared surface as a capability reached through the framework's own API.

## Framework Mapping

The framework is a project decision. Whichever one is selected, the rules of the [Desktop Standards](./standards-desktop.md) apply unchanged; the framework changes the names of the contexts, not their privileges.

| Framework | Privileged context | Unprivileged context | Declared surface |
|---|---|---|---|
| Tauri | Rust commands registered on the application builder | the web view, with the IPC allowlist the capability files declare | `#[tauri::command]` functions with typed arguments and `Result` returns |
| Iced | the `Application` update loop and its subscription layer | the view tree, which is pure data-to-element mapping | messages (`Message` enum) applied through `update` |
| Dioxus | the desktop renderer's service layer behind the asset and IPC hooks | the virtual-dom components | typed server functions or an explicit IPC bridge |
| Slint | the Rust backend callbacks registered on the component | the `.slint` UI description, whose callbacks cannot reach the OS | `#[slint::callback]` / global callbacks implemented in Rust |

- Tauri SHALL declare its IPC surface in the capability configuration and SHALL NOT enable a plugin the interface does not use. A command SHALL validate its arguments inside the command and SHALL return a typed error.
- Iced SHALL keep `update` pure: it takes the current state, one message and returns a task. A message handler SHALL NOT block, and I/O SHALL be a `Task` or a subscription.
- Dioxus SHALL NOT treat the desktop renderer as a browser: a web-only crate SHALL NOT be relied on for a capability the shipped binary must provide, and a browser target SHALL NOT be substituted for the packaged binary in an end-to-end run.
- Slint SHALL keep the UI description free of capability calls; a callback SHALL be implemented in Rust and SHALL be the only way the description reaches the platform layer.
- A framework type SHALL NOT appear in a `src/core/` signature. The framework converts its own event into the core's input type at the framework edge.
- An upgrade of the framework SHALL be treated as an interface change: the capability configuration, the declared surface and the end-to-end target SHALL be re-verified in the same change.

## Async Runtime Policy

- The project SHALL declare one async runtime — `tokio` for a Tauri application, the framework's own executor where it supplies one — and SHALL NOT nest a second runtime inside it.
- A blocking call (filesystem access, a synchronous network client, a CPU-bound loop) SHALL NOT run on an async worker thread. It SHALL be moved to a blocking pool (`tokio::task::spawn_blocking`) or to a dedicated thread.
- Every background task SHALL have an owner that can cancel it. A `tokio::spawn` whose `JoinHandle` is dropped without being stored SHALL be treated as a leak, because shutdown can no longer wait for it.
- Application shutdown SHALL cancel outstanding tasks and SHALL await their completion before the process exits, so a pending write cannot be abandoned mid-flight.
- A channel SHALL have a bounded capacity unless the producer is provably rate-limited; an unbounded channel is an unbounded memory budget.
- The interface-thread rule of the [Desktop Standards](./standards-desktop.md) SHALL be honoured in Rust terms: a synchronous call SHALL NOT block the event loop the framework renders on, and an `await` SHALL NOT be held across a lock guard.
- A timeout SHALL be declared on every await that depends on an external system, and the timeout SHALL produce a typed error rather than a silently dropped future.

## Logging and Tracing

- The project SHALL use the `tracing` facade with one subscriber installed at the binary edge. `println!` and `dbg!` MUST NOT carry operational information, and a `#[allow]` for a leftover debug macro SHALL NOT be committed.
- A span SHALL name the operation the user or the request performs (a window opened, a document loaded, a save attempted), and the span SHALL carry the correlation identifier the [Desktop Standards](./standards-desktop.md) require.
- A log event SHALL record the event and its identifiers. It MUST NOT record a credential, a token, personal data, or the full body of a file or a network response.
- The subscriber's filter SHALL be driven by one environment variable with a documented default, so a support run can raise verbosity without a rebuild, and the release build's default SHALL NOT be `trace`.
- A log file SHALL be written atomically under the namespaced user-data directory the [Desktop Standards](./standards-desktop.md) declare, and SHALL be safe to delete at any time.
- A panicking path SHALL install a panic hook that records the panic with its location before the process aborts, so a crash in the field leaves a diagnosable trail and still writes no partial user-visible state.

## Dependency and Supply-Chain Policy

- `cargo audit` SHALL run over the committed lockfile as part of the release gate. A finding at or above the project's severity threshold SHALL block the release, and an advisory accepted for a bounded time SHALL be recorded with its reason and its expiry.
- `cargo deny` SHALL run with a committed `deny.toml` and SHALL enforce four policies: advisories, licences, banned crates, and allowed registries. A crate from an unlisted registry SHALL fail the check.
- The dependency set SHALL be minimal: a crate that supplies one function the standard library or an existing dependency already provides SHALL NOT be added.
- Every dependency SHALL be declared in one place — `[dependencies]` or `[workspace.dependencies]` — with a caret requirement limited to the compatible range, and `Cargo.lock` SHALL pin the resolved version.
- A `git` dependency SHALL be pinned to a revision, and a path dependency SHALL be a workspace member or an explicitly documented local crate. A branch reference MUST NOT appear in a committed manifest.
- The third-party notice list the [Desktop Standards](./standards-desktop.md) require SHALL be generated from the lockfile by `cargo deny` or an equivalent, never assembled by hand.
- A dependency bump SHALL be a reviewable change of its own, with the lockfile diff, the advisory result and the licence result committed alongside it.

## Formatting and Linting

- `cargo fmt --all -- --check` SHALL pass on the committed tree. Formatting is not a review topic: the tool decides it.
- `cargo clippy --all-targets --all-features -- -D warnings` SHALL pass on the committed tree, and the lint configuration SHALL live in the project's own committed config (`clippy.toml` and the crate's lint attributes), not in a command a developer remembers to type.
- The crate root SHALL deny the lints the project treats as defects — `unsafe_code`, `unused_must_use`, `missing_docs` where the crate publishes an API — so a violation is a compile failure rather than a warning a reader has to notice.
- A lint that is allowed SHALL be allowed at the narrowest scope with a comment stating why. A crate-level `#![allow(...)]` without that record SHALL be a review-blocking defect.
- The lint gate and the format gate SHALL run in CI on every change, and a local pre-push hook MAY mirror them but SHALL NOT replace them.
- The scan recipe's Clippy step imports the report into the analyzer and deliberately does not fail on warnings; the developer gate above is what fails, which keeps a lint finding visible without turning analysis into a build failure.

## Testing Layers

- **Unit tests** SHALL be in-module `#[cfg(test)]` modules beside the code they cover, so a private function is testable without widening its visibility. They SHALL cover the pure logic and every error variant's construction.
- **Integration tests** SHALL live under `tests/`, one file per scenario group, and SHALL address the crate through its public surface only. `tests/` is the `sonar.tests` root, and every file there SHALL be a compiled test target.
- **Doc tests** SHALL be kept for a published API: an example in a doc comment is compiled by `cargo test` and therefore cannot rot. A doc example that needs a feature the crate does not enable SHALL be marked `ignore` with the reason.
- A test SHALL NOT reach the developer's real user-data directory. Every test that persists state SHALL create its own directory under a temporary root and remove it afterwards.
- A test that needs a platform capability SHALL use the trait seam the production code depends on, not a real keyring, not a real network socket, and not a real child process unless that process is the subject of the test.
- A defect fix SHALL land with the test that fails before the fix and passes after it, at the lowest layer that reproduces the defect.
- A flaky test SHALL be fixed or removed. A retry loop around a test SHALL NOT be added to make a suite green.
- `cargo test --all-targets` SHALL run the unit, integration, benchmark-compilation and example targets, and it SHALL be the command the mandatory unit-test step records.

## End-to-End Testing and the Driver Matrix

- The `e2e` producer's command is `cargo test --test e2e`, so **`tests/e2e.rs` is a required target**: a project that has not written it produces a loud failure with cargo's own "no test target" message, and the recorded remedy is either to write the target or to declare the `e2e` producer `NOT_APPLICABLE` with a justification in `.nsoft.yaml`.
- `tests/e2e.rs` SHALL drive the shipped application through its real entry point — the packaged binary or the framework's production launcher — and SHALL assert what the user observes: the window title, the visible state, the persisted settings file.
- The suite SHALL use an isolated user-data directory for every run and SHALL delete it afterwards. A run MUST NOT touch the developer's real application data.
- A browser build of the interface SHALL NOT substitute for the packaged binary, with one narrow exception: a Playwright run against a Dioxus **web** target MAY exist as an additional smoke check, and it SHALL NOT be recorded as the end-to-end producer's evidence.
- The driver SHALL follow the framework, and the project SHALL declare which one it uses in its guide:

| Framework | End-to-end driver | Notes |
|---|---|---|
| Tauri | WebdriverIO with `@wdio/tauri-service` | drives the packed application over the web driver protocol; `tauri-driver` is the alternative and is supported on Windows and Linux only |
| Iced | `iced_test` | drives the real `Application` through simulated interactions against the produced element tree |
| Slint | `slint::testing` | drives the component through its declared callbacks and asserts the rendered state |
| Dioxus | Playwright against the Dioxus web target | a browser-target smoke check only; the packaged binary SHALL still be exercised by a Rust-level target |

- The macOS gap SHALL be stated rather than hidden: `tauri-driver` does not support macOS, so a Tauri project targeting macOS SHALL either use the WebdriverIO service or declare that platform's coverage as a known limitation in `docs/standards-gaps.md`.
- A simulated interaction SHALL NOT be described as end-to-end. A test that calls a command handler directly is an integration test and belongs under `tests/` with the name that says so.

## Coverage and the 90 % Gate

- Coverage SHALL be measured with `cargo llvm-cov --lcov --output-path coverage/lcov.info`, which emits per-file LCOV records for the whole crate.
- Coverage SHALL hold at **90 %** for lines and functions over `src/`. The gate applies to the crate's own code: a `#[cfg(test)]` module, a generated binding and a `build.rs` output SHALL be excluded by the measurement configuration, not by lowering the threshold.
- The LCOV report SHALL be produced by the same command the `coverage` producer declares, so the file the evidence manifest points at is the file the analysis reads.
- An uncovered line SHALL be examined before it is accepted: an error branch that never runs is an untested failure path, and the fix is a test, not an exclusion. An exclusion SHALL name the reason on the line that applies it.
- Branch coverage SHALL be reviewed where the analyzer reports it; the LCOV format carries no statement counter, so the evidence file records that the counter is absent instead of inventing a number.

## SonarQube Posture

- Analysis SHALL use the native Rust analyzer, which consumes LCOV coverage and an imported Clippy report. The `javascript` coverage property SHALL NOT appear in a Rust project's configuration.
- The generated `sonar-project.properties` SHALL declare `sonar.sources=src`, `sonar.tests=tests`, `sonar.test.inclusions=tests/**/*.rs`, `sonar.exclusions=target/**`, `sonar.rust.lcov.reportPaths=coverage/lcov.info` and `sonar.rust.clippyReport.reportPaths=target/clippy-report.json`.
- The scan recipe SHALL run `cargo clippy --all-targets --message-format=json` into `target/clippy-report.json` and `cargo llvm-cov --lcov --output-path coverage/lcov.info` before the scanner starts, so both imported reports exist when the analyzer reads them.
- The recipe SHALL NOT fail the scan on a Clippy warning: the analyzer import is additive and the developer gate (`-D warnings`) is where a lint finding blocks a change.
- A missing `cargo` maps to the shipped `SCANNER_NOT_FOUND` outcome naming the tool. A present `cargo` without the `llvm-cov` subcommand surfaces cargo's own error as `SCAN_FAILED`, so the failure still names the missing subcommand.
- The analyzer version, the server version and the active rule count SHALL be recorded in the project's documentation when the analysis is first set up, so a rule-set change is a reviewable fact rather than a surprise.
- A quality-gate failure SHALL be treated as a defect in the change that introduced it, not as a threshold to relax.

## Development Workflow

- Install the pinned toolchain with `rustup`; confirm it with `cargo --version` and `rustc --version`.
- Build with `cargo build`, run with `cargo run`, and keep `Cargo.lock` committed.
- Run `cargo test --all-targets` and the coverage command before pushing, and run `cargo test --test e2e` for any change that touches the framework edge, the declared surface, the persistence layer or the packaging configuration.
- Run `cargo clippy --all-targets --all-features -- -D warnings` and `cargo fmt --all -- --check`; both are gates, and neither has an exception list.
- Run `cargo audit` and `cargo deny check` before a release and on a dependency change.
- **Branch naming**: `feature/<change-name>-desktop` or `feature/<issue-number>-<short-desc>-desktop`.
- **English only**: source, comments, error messages, log messages and commit messages are written in English.

---

This document is the Rust half of the desktop standards. The stack-neutral half states the rules every desktop stack shares; where the two meet, the neutral rule governs and this document supplies only the Rust-specific form of it.
