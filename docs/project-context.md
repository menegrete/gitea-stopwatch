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

Rust desktop application with the toolchain pinned via rustup and the edition declared in Cargo.toml. Iced 0.14.x desktop framework for multi-window, always-on-top, drag and resize with declared limits. reqwest HTTP client for the Gitea API. Gitea instance at git.nsoft.com.ar (Gitea 28.3.0, OpenAPI 3.0.3). Operating-system credential store: macOS Keychain, Windows Credential Manager, Linux Secret Service, abstracted behind a platform interface.

### Domain Conventions

Issues are identified unequivocally as owner/repo#N per ADR-02, satisfying the forge model of one issue per repository. Time entries are associated with issue comments and the sum of time linked to a comment can be queried per ADR-03. API requests authenticate with a PAT in the Authorization: token header and are rejected without a valid PAT per ADR-01/D5.

### Known Deviations from Shipped Standards

None intentional at this discovery stage. The following gaps were observed during the spikes and are tracked in docs/standards-gaps.md: Iced core does not expose role, state and descriptive label to screen readers (accessibility gap requiring AccessKit or platform-specific code), and Iced does not produce installable packages or sign binaries (packaging gap requiring external tools such as cargo-bundle).
