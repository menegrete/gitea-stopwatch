---
status: customized
source_substrate: scaffold
---
## Standards Gaps

This document records the deviations between the shipped `docs/standards-*.md` files and the actual observed patterns in your project. It is a descriptive inventory of gaps, not a normative policy document. Where a deviation is intentional, record it here along with its justification. Where it is unintentional, use this record to plan remediation.

> **Important**: Passing `--force` during `init` will overwrite this file, discarding any hand-filled content. If you rely on this file, preserve a backup before running `init --force`.

### Deviations from Shipped Standards

**Gaps from standards-desktop.md - Accessibility and Desktop UI Conventions.** The standard requires every interactive control to expose an accessible name and role so a screen reader can announce it. Iced core does not expose role, state or descriptive label to screen readers in a complete way; the iced-desktop spike confirmed this limitation. Remediation requires a third-party crate such as Plushie Iced with AccessKit integration or platform-specific code. Unintentional, planned for remediation.

**Gaps from standards-desktop.md - Packaging and Distribution.** The standard requires a release artifact produced by a reproducible build, code-signed and notarized where the platform requires it. Iced only produces an executable; it does not generate installable packages (.deb, .rpm, .msi, .dmg) or sign binaries. Packaging and signing are the responsibility of external tools such as cargo-bundle or the OS packager. Unintentional, planned for remediation.

**Gaps from standards-desktop-rust.md - Framework Mapping.** The framework mapping table includes Iced with the Application update loop as privileged context and the view tree as unprivileged context. The spike confirmed that Iced multi-window, always-on-top, drag and resize capabilities compile and run, but accessibility and packaging are not natively supported. This gap is captured in the accessibility and packaging entries above.

**Gaps from standards-desktop.md - Local State and User Data / Security Posture.** Credentials, tokens and key material must be stored in the operating system's credential store and must not be written to a plain file. ADR-04 defines encrypted credential storage in the OS keystore with platform-specific implementations (macOS Keychain, Windows Credential Manager, Linux Secret Service). The abstraction adds cross-platform complexity. Intentional deviation accepted for security posture; complexity is the trade-off.

**Gaps from openspec-tasks-mandatory-steps.md / standards-desktop-rust.md - End-to-End Testing.** No `tests/e2e.rs` target exists in this change; the change delivers no product code under src/. The e2e producer is recorded not applicable with justification. This will be revisited when implementation begins.
