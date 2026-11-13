---
status: customized
source_substrate: scaffold
---
## Standards Gaps

This document records the deviations between the shipped `docs/standards-*.md` files and the actual observed patterns in your project. It is a descriptive inventory of gaps, not a normative policy document. Where a deviation is intentional, record it here along with its justification. Where it is unintentional, use this record to plan remediation.

> **Important**: Passing `--force` during `init` will overwrite this file, discarding any hand-filled content. If you rely on this file, preserve a backup before running `init --force`.

### Deviations from Shipped Standards

**Gaps from standards-desktop.md - Accessibility and Desktop UI Conventions.** The standard requires every interactive control to expose an accessible name and role so a screen reader can announce it. Iced core does not expose role, state or descriptive label to screen readers in a complete way; the iced-desktop spike confirmed this limitation. Remediation requires a third-party crate such as Plushie Iced with AccessKit integration or platform-specific code. Unintentional, planned for remediation.

**Gaps from standards-desktop.md - Packaging and Distribution.** The standard requires a release artifact produced by a reproducible build, code-signed and notarized where the platform requires it. Iced only produces an executable; it does not generate installable packages (.deb, .rpm, .msi, .dmg) or sign binaries. Packaging and signing are the responsibility of external tools such as `cargo-bundle` or the OS packager. Unintentional, planned for remediation.

**Gaps from standards-desktop-rust.md - Framework Mapping.** The framework mapping table includes Iced with the Application update loop as privileged context and the view tree as unprivileged context. Iced multi-window, always-on-top, drag and resize capabilities compile and run; accessibility and packaging are not natively supported. This gap is captured in the accessibility and packaging entries above.

**Gaps from standards-desktop.md - Local State and User Data / Security Posture.** Credentials, tokens and key material must be stored in the operating system's credential store and must not be written to a plain file. Encrypted credential storage in the OS keystore with platform-specific implementations (macOS Keychain, Windows Credential Manager, Linux Secret Service). The abstraction adds cross-platform complexity. Intentional deviation accepted for security posture; complexity is the trade-off.

**Gaps from openspec-tasks-mandatory-steps.md / standards-desktop-rust.md - End-to-End Testing.** `tests/e2e.rs` exists and runs against an isolated user-data directory, cleaning up after execution. The e2e producer is applicable and passes. One test executes the shipped binary (resolved through `CARGO_BIN_EXE_gitea-stopwatch`) under a resolved display harness (`DISPLAY` when reachable, otherwise `xvfb-run -a`), drives clean shutdown through the documented `GITEA_STOPWATCH_E2E_AUTO_CLOSE_MS` harness, and asserts the startup log line, the mapped window title, exit code `0`, lock release, persisted geometry and the absence of panics. Two limitations remain and are not hidden: the rendered theme and the placeholder-navigation text are asserted through the crate's public constants rather than read back from the window, because a headless X server exposes no readable frame content; and the test skips with a printed reason on runners that have neither a reachable `DISPLAY` nor `xvfb-run` (Wayland-only sessions and macOS/Windows CI runners), so execution of the binary is not guaranteed on every platform. Unintentional, planned for remediation when a GUI-driving harness (window-manager interaction or pixel capture) is adopted.

**Coverage.** `cargo llvm-cov --lcov` produces the LCOV report for the crate's own code under `src/`. The 90 % line/function baseline is recorded as a trend (not a hard CI gate) until feature work begins, per the test plan. The Iced view layer (`ui/window.rs`) and binary edge (`main.rs`, `lib.rs::run`) are not exercised by unit tests because they require a display server; coverage over pure logic (`core/`, `platform/`) measures 96.49 % lines and 92.86 % functions. The SonarQube quality gate consequently reports `PASS` without a coverage condition on its server-side configuration; the rationale is recorded under §SonarQube Quality Gate of `docs/project-context.md` and in `coverage-baseline.json`. Unintentional, planned for remediation when feature work begins and the gate gains its coverage condition.
