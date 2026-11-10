---
description: Stack-neutral desktop application standards — process and trust boundaries, typed inter-process contracts, application lifecycle, local state, packaging and distribution, security posture, accessibility, performance budgets and the testing expectations that apply to every desktop stack
globs: ["src/**/*.ts", "src/**/*.tsx", "test/**/*.test.ts", "package.json", "tsconfig.json"]
alwaysApply: true
---

# Desktop Project Configuration and Best Practices

## Table of Contents

- [Overview](#overview)
- [Scope and Relationship to the Stack Standards](#scope-and-relationship-to-the-stack-standards)
- [Process and Trust Boundaries](#process-and-trust-boundaries)
- [Inter-Process Contracts](#inter-process-contracts)
- [Application Lifecycle](#application-lifecycle)
- [Local State and User Data](#local-state-and-user-data)
- [Packaging and Distribution](#packaging-and-distribution)
- [Security Posture](#security-posture)
- [Accessibility and Desktop UI Conventions](#accessibility-and-desktop-ui-conventions)
- [Performance Budgets](#performance-budgets)
- [Testing Standards](#testing-standards)
- [Development Workflow](#development-workflow)

---

## Overview

This document defines the normative standard for a **desktop application**: software installed and executed on an end user's machine. Every rule below holds whichever stack the project selects, and none of them depends on the technology that implements the desktop shell. The stack-specific companion standard refines these rules with its technology's own conventions and SHALL NOT contradict them; where that companion is silent, this document governs.

Read this document together with the [Core Standards](./standards-core.md) and the [Documentation Standards](./standards-documentation.md); the repository workflow and its mandatory steps are described in the [Development Guide](./development-guide.md).

## Scope and Relationship to the Stack Standards

- This document SHALL be installed for every desktop Application Type, whatever stack the project declares.
- The stack-specific desktop standard SHALL carry only the rules that follow from the selected technology. It SHALL NOT restate, relax or override a rule stated here.
- A rule that depends on the chosen technology — the concrete process model, the runtime APIs, the packaging tool — SHALL live in the stack-specific standard and MUST NOT be duplicated here.
- The two documents SHALL be maintained as a normative pair: a change to a shared rule lands here first, and the stack-specific standard then refers to it instead of restating it.
- A project that selects a stack for which no stack-specific standard has shipped yet SHALL follow this document alone, and SHALL record the missing stack rules in `docs/standards-gaps.md`.

## Process and Trust Boundaries

- A desktop application SHALL separate a **privileged context**, which owns the operating-system capabilities, from an **unprivileged UI context**, which renders the interface and handles user input.
- The privileged context SHALL own every capability that reaches outside the application's own memory: filesystem access, network sockets, child processes, native menus and tray integration, launch-at-login registration, and credential storage.
- The unprivileged UI context SHALL run with the least privilege the interface needs. It MUST NOT read the filesystem, spawn a process, or read a credential directly.
- Every privileged capability the interface needs SHALL be reachable only through a declared, minimal surface, as required by [Inter-Process Contracts](#inter-process-contracts). A capability that no declared surface exposes SHALL NOT be reachable at all.
- The privileged context SHALL own window and application lifecycle: window creation, focus, persisted window geometry, and orderly shutdown.
- A privileged context SHALL NOT execute code fetched at runtime from the network. Every line of application code SHALL ship inside the installed package.
- Security-relevant settings SHALL be restrictive by default and SHALL be relaxed only by an explicit, reviewed configuration entry that states its reason in a comment next to the entry.
- The boundary SHALL be asserted by a test: at least one integration test SHALL prove that the unprivileged context cannot reach a privileged capability it was not granted.

## Inter-Process Contracts

- Every call across the trust boundary SHALL be declared in one shared contract module that both sides import. The contract SHALL type the request, the response, and every error variant it can return.
- Message identifiers SHALL be declared once, in that module, following a single namespaced pattern `<domain>:<action>`. A string literal typed at a call site SHALL NOT be used as an identifier.
- Request/response SHALL be the default shape. A one-way message SHALL be used only for an event the receiver cannot answer, and the contract SHALL mark it as one-way.
- The privileged side SHALL validate every argument it receives before acting on it, and SHALL return a typed failure instead of throwing an untyped value across the boundary.
- The unprivileged side SHALL receive a narrow, named API — one function per declared operation — and MUST NOT receive a generic transport object that can address arbitrary messages.
- A contract change SHALL be treated as a versioned interface change: both sides SHALL be updated in the same change, and adding a message SHALL touch the contract module and no other shared file.
- The contract module SHALL be pure: no runtime import of the privileged context, and no side effect at import time, so both sides and the unit tests can load it.
- The contract module SHALL be unit-tested on its own, including the rejection path for every validated argument.

## Application Lifecycle

- The application SHALL acquire a single-instance lock before it opens its main window. A second launch SHALL hand its arguments to the running instance and exit without opening a second window.
- Startup SHALL create no window before the application is ready to render, and SHALL NOT block the first render on a network request.
- Shutdown SHALL be graceful and idempotent: pending writes complete or are abandoned deliberately, and a repeated shutdown request SHALL NOT corrupt persisted state.
- The application SHALL persist and restore window geometry, and SHALL clamp restored geometry to the displays that are attached at launch time.
- A migration of persisted state SHALL run once, SHALL be keyed by a stored schema version, and SHALL be forward-only. State written by a newer application version SHALL be refused with a diagnostic rather than silently downgraded.
- Offline behaviour SHALL be defined for every feature that reads remote data: the feature SHALL show the last known local state with a clear staleness indicator, and SHALL NOT fail silently.
- A crash SHALL leave no partially written user-visible state; the next launch SHALL recover to the last complete state and SHALL record the recovery.

## Local State and User Data

- The user-data directory SHALL be resolved through the platform's per-user application-data location. It MUST NOT be derived from the process working directory and MUST NOT be a hardcoded absolute path.
- A write to a persisted file SHALL be atomic — write a temporary file, then rename — so an interrupted write cannot truncate the previous state.
- Credentials, tokens and key material SHALL be stored in the operating system's credential store. They MUST NOT be written to a plain file, a log entry, an environment default, or a crash report.
- A log entry SHALL record the event and its correlation identifier. It MUST NOT record credentials, tokens, personal data, or full file contents.
- Cache and scratch directories SHALL be namespaced by application name and version, and SHALL be safe to delete at any time without losing user-visible state.
- A test SHALL create its own user-data directory and remove it afterwards. A test MUST NOT read or write the developer's real application data.

## Packaging and Distribution

- A release artifact SHALL be produced by a reproducible build from a committed configuration. A generated installer, bundle or archive MUST NOT be hand-edited; the configuration is the source of truth.
- The application version SHALL come from one declared source and SHALL be reported consistently by the window title, the about surface, the artifact name, and the update channel.
- A distributed build SHALL be code-signed, and SHALL be notarized where the target platform requires it. An unsigned artifact SHALL NOT be published to end users.
- An update SHALL be delivered over a channel whose payload signature is verified before installation, and a failed update SHALL leave a usable installation behind.
- Every artifact SHALL declare the operating systems and architectures it supports. An artifact that was not built for a platform SHALL NOT be offered to it.
- The packaged application SHALL carry the licence, the third-party notice list, and the versions of the bundled dependencies that the project's policy requires.
- The packaging configuration SHALL be reviewed like source code, and a change to it SHALL be exercised by the release pipeline before it reaches end users.

## Security Posture

- The dependency set SHALL be audited for known vulnerabilities as part of the release gate, and a finding at or above the project's severity threshold SHALL block the release.
- Secret scanning SHALL run over the committed tree. A credential found in the repository SHALL be rotated, not merely deleted.
- The application SHALL deny by default the platform capabilities it does not need: camera, microphone, geolocation, notifications, clipboard writes, and external protocol launches.
- Navigation away from the application's own origin SHALL be denied, or SHALL be opened in the user's default browser after an explicit user action. The privileged context MUST NOT navigate to an arbitrary remote origin.
- Remote content SHALL be subject to a declared content security policy, and the policy SHALL be the restrictive one the interface actually needs.
- A URL received from the network or from a document SHALL be validated against an allowlist of schemes before it is opened.
- A privilege escalation path discovered in review SHALL be treated as a blocking defect, not as a follow-up improvement.

## Accessibility and Desktop UI Conventions

- Every interactive control SHALL be reachable and operable by keyboard alone, in a logical focus order, and the focused control SHALL be visibly focused.
- The interface SHALL expose an accessible name and role for every control so a screen reader can announce it.
- Colour SHALL NOT be the only carrier of meaning; every state SHALL carry a textual or structural cue as well.
- The interface SHALL honour the platform's high-contrast and reduced-motion settings.
- A long-running operation SHALL report progress and SHALL remain cancellable.
- A destructive action SHALL require confirmation and SHALL name the object it affects.

## Performance Budgets

- The project SHALL declare a cold-start budget and a steady-state memory budget, and every release SHALL be measured against both.
- Work that the first screen does not need SHALL be deferred until the interface is interactive.
- A synchronous call across the trust boundary SHALL NOT appear on the interface path.
- A list that can grow without bound SHALL be virtualized.
- A regression against a declared budget SHALL be treated as a defect, not as an accepted cost of the feature that caused it.

## Testing Standards

- Unit tests SHALL cover the pure logic and the contract module, and SHALL run without launching the desktop shell.
- An integration test SHALL exercise at least one call across the real trust boundary, with the real contract module on both sides.
- End-to-end tests SHALL drive the shipped application through its real entry point, never a browser-only build of the interface, and SHALL assert what the user observes: window title, visible state, and persisted files.
- Test files SHALL live in the test tree the stack standard declares and mirror the source tree it names, so the analysis keeps sources and tests separated: the npm convention is one instance, with `test/` and `*.test.ts`/`*.spec.ts`, and a compiled stack names its own test root and extensions in its stack standard.
- Coverage SHALL hold at **90%** for branches, functions, lines and statements.
- A defect fix SHALL land with the test that fails before the fix and passes after it.
- A flaky end-to-end test SHALL be fixed or removed; it SHALL NOT be retried until it passes.

## Development Workflow

- **Feature branches**: `feature/<change-name>-desktop` or `feature/<issue-number>-<short-desc>-desktop`, so desktop work stays parallel to the other Application Types.
- **Descriptive commits**: Conventional Commits written in English.
- **Code review**: required before merge.
- **Quality gates**: lint, type-check, unit tests, coverage and the end-to-end suite SHALL pass before a pull request is opened.
- **English only**: source, comments, error messages and log messages are written in English.

---

This document is the stack-neutral half of the desktop standards. The stack-specific standard adds the selected technology's own rules without restating these.
