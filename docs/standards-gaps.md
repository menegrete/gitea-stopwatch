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

**Gaps from openspec-tasks-mandatory-steps.md / standards-desktop-rust.md - End-to-End Testing.** `tests/e2e.rs` exists and runs against an isolated user-data directory, cleaning up after execution. The e2e producer is applicable and passes. E2E drives through the crate's public surface (`dirs`, `APP_TITLE`) rather than the packed Iced binary (no display server available in this environment); this is a known limitation noted for CI runners with a display server.

**Coverage.** `cargo llvm-cov --lcov` produces the LCOV report for the crate's own code under `src/`. The 90 % line/function baseline is recorded as a trend (not a hard CI gate) until feature work begins, per the test plan. The Iced view layer (`ui/window.rs`) and binary edge (`main.rs`, `lib.rs::run`) are not exercised by unit tests because they require a display server; coverage over pure logic (`core/`, `platform/`) is at or near 100 %.
