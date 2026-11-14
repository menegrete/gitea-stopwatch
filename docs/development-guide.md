# Development Guide (Template)

This guide provides a reusable setup and day-to-day development workflow template for a Rust desktop application.

Replace all placeholder values in angle brackets before using this guide in a real project.

## 1. References

- [Core Standards](./standards-core.md)
- [Desktop Standards](./standards-desktop.md)
- [Desktop Rust Standards](./standards-desktop-rust.md)
- [Documentation Standards](./standards-documentation.md)

## 2. Placeholder Conventions

Use these placeholder patterns across all project documentation:

- `<PROJECT_NAME>`
- `<REPO_URL>`
- `<DESKTOP_PATH>` — the application root, resolved from `scanRoot` in `nsoft-project.properties` (written by `nsoft-specboot init`)
- `<APP_ID>` (example: `com.example.desktop`)
- `<API_BASE_URL>`
- `<FEATURE_BRANCH_PREFIX>` (example: `feature`)
- `<FRAMEWORK>` — the selected desktop framework (`Tauri`, `Iced`, `Dioxus` or `Slint`)

## 3. Prerequisites

Install the required tooling versions for your project:

- `Git` >= `<MIN_GIT_VERSION>`
- `rustup` with the pinned toolchain: `rustup toolchain install` reads `rust-toolchain.toml`, so the channel, edition support and the `rustfmt`/`clippy` components all come from the committed file
- The coverage subcommand: `rustup component add llvm-tools-preview` followed by `cargo install cargo-llvm-cov --locked`. Both are prerequisites of the coverage gate; a missing subcommand is reported as a missing prerequisite, never installed mid-run
- The supply-chain subcommands: `cargo install cargo-audit --version 0.23.0 --locked` and `cargo install cargo-deny --version 0.20.2 --locked`, the same pinned versions and `--locked` flag the CI workflow installs
- The framework toolchain for `<FRAMEWORK>`, as the framework's own documentation requires for a release build
- A platform toolchain for the targets you build: the platform SDK for code signing, and the notarization credentials where the platform requires them

## 4. Clone and Bootstrap

```bash
git clone <REPO_URL>
cd <PROJECT_NAME>
```

## 5. Environment Configuration

Create `<DESKTOP_PATH>/.env` (never committed) or export the values in your shell:

```text
APP_ENV=development
API_BASE_URL=<API_BASE_URL>
RUST_LOG=info
```

- Configuration and credentials are read at the privileged edge only. A secret MUST NOT be compiled into the shipped binary.
- Credentials the application uses at runtime belong in the operating system's credential store, not in `.env` and not in a committed file.
- `RUST_LOG` raises verbosity for a local run; the release default is documented in the stack standard and is not `trace`.

## 6. Build and Run

From `<DESKTOP_PATH>`:

```bash
cargo build
cargo run
```

Optional commands:

```bash
cargo build --release
cargo build --all-targets   # compiles unit, integration, bench and example targets
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

`cargo run` starts the application directly: there is no browser-served interface and no separate server process. The window is the application.

## 7. Testing Workflow (`cargo test`)

From `<DESKTOP_PATH>`:

```bash
cargo test --all-targets
```

Requirements:

- Unit tests live in `#[cfg(test)]` modules beside the code they cover; integration tests live in `tests/`, one file per scenario group, and `cargo test --all-targets` compiles every one of them
- Doc tests are part of the same run, so an example in a doc comment cannot rot
- Cover the success path and the failure path, and cover the rejected-argument path of every operation the interface reaches
- Test the core logic against an injected capability; never against a real keyring, a real network socket or a launched window
- Maintain a coverage threshold of `90%` for lines and functions over the unit-testable scope (`src/core/`, `src/platform/`); the display-bound files are covered by the end-to-end target and the overall percentage is a trend (see §8)
- A test SHALL create its own user-data directory under a temporary root and remove it afterwards; it MUST NOT touch the developer's real application data

This project follows the universal `src/` + `tests/` convention: application code lives in `src/`, tests live in `tests/`, so the generated `sonar-project.properties` declares `sonar.sources=src` and `sonar.tests=tests` with `sonar.test.inclusions=tests/**/*.rs`.

## 8. Coverage Gate (`cargo llvm-cov`)

From `<DESKTOP_PATH>`:

```bash
LLVM_COV_FLAGS='-ignore-filename-regex=(^|[\\/])src[\\/]((main|lib)\.rs|ui[\\/])' \
  cargo llvm-cov --lcov --output-path coverage/lcov.info \
  --fail-under-lines 90 --fail-under-functions 90
```

Requirements:

- The gate applies to the unit-testable scope `src/core/` and `src/platform/`; `src/ui/`, `src/lib.rs` and `src/main.rs` require a display server and are covered by the end-to-end target instead. The `LLVM_COV_FLAGS` value above is the form recorded in `coverage-baseline.json`
- The `[\\/]` character classes are separator-agnostic on purpose: they match the forward slash `llvm-cov` emits on Linux and the backslash it emits on Windows. A POSIX-only `(^|/)src/((main|lib)\.rs|ui/)` regex matches the same set on Linux but would miss `src\ui\...` paths on Windows
- The `coverage` producer records the bare `cargo llvm-cov --lcov --output-path coverage/lcov.info` command, so the scope travels in the exported variable and reproducing the gate requires the invocation above
- The report is LCOV: it carries per-file line and function counters and no statement counter, and the evidence file records that the statement counter is absent instead of inventing a number
- `branches` reads `100` over a `0/0` denominator on this crate and is vacuous: the scoped records carry `BRF:0`/`BRH:0`, so branch coverage is not measured and only lines and functions are gated
- The threshold is `90%` for lines and functions over the unit-testable scope; a `#[cfg(test)]` module and a generated binding are excluded by configuration, not by lowering the threshold
- An uncovered line is examined before it is accepted: an error branch that never runs is an untested failure path, and the fix is a test rather than an exclusion

## 9. SonarQube Deterministic Scans

### Prerequisites

The scan runs on the machine that performs the review and requires the following toolchain:

- **Bash** — Git Bash on Windows, native bash on Linux/macOS (required when the scan uses the `.sh` script variant)
- **PowerShell** (Windows only) — PowerShell 5.1+, used for the native `.ps1` scan scripts; on Windows the skill invokes `.ps1` natively, so Git Bash is not required for local scans
- **SonarScanner CLI** (`sonar-scanner` on PATH) — the core scanner binary. Install from the [SonarScanner CLI docs](https://docs.sonarsource.com/sonarqube/latest/analyzing-source-code/scanners/sonarscanner/) and add the `bin/` directory to your `PATH`. Verify with `sonar-scanner --version`
- **curl** — used for quality-gate polling (included with Git Bash on Windows; pre-installed on most Linux/macOS distributions)
- **Standard POSIX utilities** — `grep`, `sed`, `head`, `cut`, `tr`, `basename`, `date`, `sleep`, used for output parsing (all included with Git Bash on Windows; part of coreutils on Linux/macOS)
- **`cargo`** — the Rust scan recipe runs Clippy and the coverage command to produce the two imported reports; verify with `cargo --version`
- **`cargo-llvm-cov`** — the coverage subcommand; verify with `cargo llvm-cov --version`. A `cargo` present without this subcommand fails the scan with cargo's own error naming `llvm-cov`

**Environment variables:**

- `SONAR_TOKEN` or `OPENCODE_SONARQUBE_TOKEN` — SonarQube authentication token (generate from your SonarQube server under **My Account > Security > Generate Token**)
- `OPENCODE_SONARQUBE_URL` — SonarQube server URL (or pass via `--sonar-url`)
- `SONAR_MAX_ATTEMPTS` — quality gate poll retries (default: 30)
- `SONAR_POLL_INTERVAL` — poll interval in seconds (default: 5)
- `SONAR_POST_ANALYSIS_DELAY` — seconds to wait for SonarQube to process the analysis (default: 20)

**Project files:**

- `sonar-project.properties` — required at the project root, generated by `nsoft-specboot init`.

### sonar-project.properties generation

`nsoft-specboot init` writes `sonar-project.properties` at the project root. For this Application Type with the `rust` stack it declares `sonar.sources=src`, `sonar.tests=tests`, the test inclusions `tests/**/*.rs`, the exclusion `target/**`, the coverage report path `sonar.rust.lcov.reportPaths=coverage/lcov.info` and the Clippy report path `sonar.rust.clippyReport.reportPaths=target/clippy-report.json`. It carries no `sonar.javascript.lcov.reportPaths` entry, because a Rust project's coverage is read by the native Rust analyzer. Re-running `init` preserves consumer customizations (additive merge) and updates only the section matching the detected application type and resolved stack.

### Scan recipe via nssx-review

`nssx-review` runs the scan through the `nsoft-specboot scan` CLI, which performs the scan pipeline natively inside the CLI process with no scan-script delegation:

1. Run `nsoft-specboot scan` from the git repository root with zero arguments.
2. The CLI detects the application type and the resolved stack from `nsoft-project.properties` and executes the stack's scan pipeline in-CLI. The `rust` stack resolves the `rust` pipeline: it runs `cargo clippy --all-targets --message-format=json` into `target/clippy-report.json` and `cargo llvm-cov --lcov --output-path coverage/lcov.info`, then maps both report paths to the analyzer. The Clippy step deliberately does not fail the scan on a warning; the `-D warnings` gate below is where a lint finding blocks a change.
3. Read the single JSON object on stdout (`scanStatus`, `qualityGate`, `projectKey`, `analysisTimestamp`) and fetch the quality gate via the SonarQube MCP.
4. Record the scan and quality gate status (`PASSED`/`FAILED`/`SKIPPED`) in `review.md`.

The build output (`target/`) is excluded from analysis; only `src/` is analyzed and only `tests/` provides tests.

## 10. End-to-End Testing

The end-to-end producer runs one project-owned cargo target, and it evidences the mandatory step **End-to-end testing of the shipped desktop application (MANDATORY)**:

```bash
cargo test --test e2e
```

Requirements:

- **`tests/e2e.rs` is a required target.** A crate without it produces cargo's own "no test target" error, recorded as a `FAIL` with that output. The documented alternative is an explicit `producers_not_applicable` declaration for the `e2e` producer with a justification, never a fabricated pass
- The target drives the shipped application through its real entry point and asserts what the user observes — window title, visible state, persisted settings file — and not only that the process started
- Pass an isolated user-data directory under a temporary path for every run, and delete it afterwards; the suite must never touch the developer's real application data
- Do not start a development server and do not substitute a browser build of the interface for the packaged binary. The single exception is a Dioxus web-target Playwright run, which is an additional smoke check and is never recorded as the end-to-end evidence
- The driver follows the framework: WebdriverIO with `@wdio/tauri-service` for Tauri (`tauri-driver` is the alternative, supported on Windows and Linux only), `iced_test` for Iced, `slint::testing` for Slint, and Playwright only against a Dioxus web target. A simulated interaction is an integration test, not an end-to-end test

## 11. Branching and Commit Conventions

Use small, focused branches and descriptive English commit messages.

Recommended branch naming:

- Desktop: `<FEATURE_BRANCH_PREFIX>/<CHANGE_NAME>-desktop`

## 12. Code Quality Gates

Before opening a pull request, ensure:

- `cargo build --all-targets` succeeds with no warning
- `cargo fmt --check` passes
- `cargo clippy --all-targets --all-features -- -D warnings` passes
- `cargo test --all-targets` passes
- Coverage meets the `90%` threshold on lines and functions
- `cargo test --test e2e` passes for any change touching the framework edge, the declared surface, the persistence layer or the packaging configuration
- `cargo audit` and `cargo deny check` pass, and the third-party notice list is regenerated when the dependency set changed
- Required docs are updated

## 13. Pull Request Checklist (Template)

- [ ] Environment placeholders were replaced for the target project
- [ ] Desktop standards were followed (process boundary, typed surface, local state, security posture)
- [ ] Desktop Rust standards were followed (error boundaries, the unsafe policy, the async runtime, the supply-chain policy, the lint and format gates)
- [ ] Every new privileged capability is declared once, reachable only through the declared surface, and covered by a test
- [ ] No `unsafe` block was added without its `// SAFETY:` argument, or the crate still carries `#![forbid(unsafe_code)]`
- [ ] `tests/e2e.rs` was added or updated for a change touching user-visible behaviour, or the `e2e` producer is explicitly declared not applicable with a justification
- [ ] Tests added/updated and passing locally, with the isolated user-data directory removed
- [ ] Packaging configuration reviewed if the artifact layout changed
- [ ] `Cargo.lock` and the third-party notice list committed with the dependency change
- [ ] Documentation updated (guide/readme/api as needed)
- [ ] No secrets committed and no signing identity in the tree

## 14. Troubleshooting (Template)

### 14.1 The Window Opens Blank

- Confirm the binary was built after the interface assets changed; a stale build is the most common cause
- Check the application log for a failure while loading the initial view

### 14.2 The Application Cannot Reach the API

- Validate `<API_BASE_URL>` in the privileged configuration
- Remember that the interface cannot call the network directly: the call belongs in the platform layer, behind a declared operation

### 14.3 `cargo llvm-cov` Is Not Found

- Add the `llvm-tools-preview` component and install the subcommand with `cargo install cargo-llvm-cov --locked`
- Confirm the subcommand resolves with `cargo llvm-cov --version`; a missing subcommand fails the scan with cargo's own error naming `llvm-cov`

### 14.4 Packaging Fails on Signing

- Confirm the signing identity and its password are present in the CI environment for the target platform
- Confirm the notarization credentials are present where the platform requires notarization

## 15. Project-Specific Overrides

Document project-specific deviations from this template here:

- `<OVERRIDE_1>`
- `<OVERRIDE_2>`
- `<OVERRIDE_3>`

## Apply-Phase Evidence Capture and Template Sync

### Apply-Phase Evidence Convention

Every agent-executed validation step during the apply phase leaves deterministic, machine-parseable, redacted evidence under `openspec/changes/<name>/logs/`, captured with the canonical command:

```bash
nsoft-specboot collect <change-name>
```

The subcommand ships with the package, resolves the change root internally, and requires no path argument. It runs this Application Type's automated-test producers and writes the `logs/evidence.json` manifest recording each producer's command, exit code, PASS/FAIL outcome, and coverage metrics against their thresholds. Run it once after the automated-test steps complete; verify exit code 0 and `RESULT: PASS`. A non-zero exit means a producer failed or capture failed — the evidence is still written, so the failure is evidenced, not hidden; fix and re-run. The subcommand's child processes redirect the full global surface (`HOME`, `USERPROFILE`, `XDG_CONFIG_HOME`, `XDG_CACHE_HOME`, `APPDATA`, `LOCALAPPDATA`) so the developer's real global surface is never touched, and the run is hard-gated with a SHA-256 byte-identity fingerprint taken before and after.

### Reviewer-Side Reading

The reviewer reads `openspec/changes/<name>/logs/` and cross-checks each producer entry against the `tasks.md` step it evidences. Evidence present and consistent with the claim verifies the step; missing or contradictory evidence for a MANDATORY validation step is a Blocker that forces `VERDICT: FAIL`; missing evidence for a non-mandatory step is a Warning that does not affect the verdict. The reviewer never silently assumes a step ran.

### Template Sync

The template mirror `template/.ai-specs/` is kept aligned with the canonical `.ai-specs/` source: `npm run sync-template-ai-specs` applies the sync with add/overwrite/delete semantics, and `npm run sync-template-ai-specs:check` is the read-only verification that exits 0 when aligned and non-zero with a divergence list when stale. The sync script is tooling of the scaffolding repository; an installed Rust project consumes the synced specs and does not need `npm` to build or test its own crate.
