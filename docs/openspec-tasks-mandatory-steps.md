---
description: Enforce mandatory steps from openspec/config.yaml when creating tasks.md artifacts for a Rust desktop Application Type
alwaysApply: true
---

# OpenSpec Tasks: Mandatory Steps Enforcement (Desktop Rust)

When creating or updating `tasks.md` artifacts in OpenSpec changes, you MUST:

## 1. Read openspec/config.yaml First

**BEFORE** creating or updating any `tasks.md` file, you MUST read `openspec/config.yaml` to understand branch naming conventions, task structure requirements, and testing/documentation requirements.

## 2. Mandatory Steps

All implementation tasks MUST include these steps in the correct order:

1. **Step 0 (must be first)**: Create and switch to feature branch:
   - `feature/<issue-number>-<short-desc>` or `feature/<change-name>`
2. **Review and update existing unit tests (MANDATORY)**
3. **Run unit tests and coverage checks (MANDATORY)**
4. **End-to-end testing of the shipped desktop application (MANDATORY)** — AGENT MUST EXECUTE
5. **Update technical documentation (MANDATORY)**

## 3. Manual Testing Requirements - CRITICAL: Agent Must Execute

**IMPORTANT**: The coding agent MUST execute the end-to-end target itself with `cargo test --test e2e`. **NEVER delegate testing to the user**.

### Step 4: End-to-end testing of the shipped desktop application (MANDATORY)

**When This Applies**: Any change affecting a user-facing workflow, the framework edge, the declared capability surface, the persistence layer or the packaged artifact.

**Implementation Steps** (Agent must perform):
1. Build the application with `cargo build` so the binary under test is the one just produced; the end-to-end run launches the application itself, so there is no development server and no browser target to substitute
2. Run `cargo test --test e2e` and confirm the target exists; a crate without `tests/e2e.rs` fails with cargo's own "no test target" error, and the only accepted remedy is an explicit `producers_not_applicable` declaration for `e2e` with a justification recorded in `.nsoft.yaml`
3. Exercise the complete user workflow through the framework's declared driver, driving the shipped entry point: WebdriverIO with `@wdio/tauri-service` (or `tauri-driver`, Windows and Linux only) for Tauri, `iced_test` for Iced, `slint::testing` for Slint, and Playwright only against a Dioxus web target as an additional smoke check
4. Test validation and error scenarios, and assert that the result a capability call returns is what the interface shows
5. Verify the window title, the visible state, and the persisted state file match the expected outcome at each step, using an isolated user-data directory under a temporary path
6. Delete the isolated user-data directory and any driver artifact created for the run
7. Mark the task as completed only after the whole target passes

## 4. Verification Checklist

Before finalizing any `tasks.md` file, verify:
- [ ] Step 0 (Create Feature Branch) is the FIRST step
- [ ] All mandatory steps above are included
- [ ] Steps are numbered sequentially
- [ ] Branch naming follows the convention: `feature/[name]-desktop`
- [ ] The end-to-end step is named exactly as the `e2e` producer's `step` and explicitly states "AGENT MUST EXECUTE"
- [ ] Evidence-presence item: `logs/evidence.json` exists under the change folder's `logs/` recording the `cargo test`, the `cargo llvm-cov` coverage run and the end-to-end target with exit codes and PASS/FAIL outcomes
- [ ] Documentation update step is included

## 5. When This Applies

This rule applies when creating or updating `tasks.md` for a repository whose Application Type is `desktop` and whose resolved stack is `rust`.

## 6. Example Structure

```markdown
## 0. Setup: Create Feature Branch (MANDATORY - FIRST STEP)

- [ ] 0.1 Create feature branch `feature/update-settings-window-desktop` from main/master branch
- [ ] 0.2 Verify branch creation and current branch status

## 11. End-to-end testing of the shipped desktop application (MANDATORY - AGENT MUST EXECUTE)
- [ ] 11.1 Build the crate with `cargo build` and confirm the binary under test exists
- [ ] 11.2 Confirm `tests/e2e.rs` exists, or record the `e2e` producer as not applicable with its justification
- [ ] 11.3 Run `cargo test --test e2e` against an isolated user-data directory
- [ ] 11.4 Execute the complete user workflow through the framework's declared driver
- [ ] 11.5 Test error scenarios and validation, asserting the capability result shown in the interface
- [ ] 11.6 Verify the window title, UI state, and persisted state file
- [ ] 11.7 Remove the isolated user-data directory and document the scenarios and outcomes

## 16. Update Technical Documentation (MANDATORY)
...
```

## 7. Agent Execution Requirements

**CRITICAL**: The coding agent MUST execute the end-to-end target itself with `cargo test --test e2e`, never ask the user to run it, and mark tasks as completed only after successful execution and documentation.

## 8. Apply-Phase Evidence Capture

During the apply phase, capture deterministic evidence of every automated test execution under `openspec/changes/<name>/logs/` using the canonical command:

```bash
nsoft-specboot collect <change-name>
```

The `collect` subcommand ships with the package, resolves the change root internally, and requires no path argument. It runs this Application Type's automated-test producers — `cargo test` for the unit tests, `cargo llvm-cov --lcov --output-path coverage/lcov.info` for the coverage metrics, and `cargo test --test e2e` for the end-to-end target — resolved internally by the subcommand, and records each command, its exit code, and the PASS/FAIL outcome in the `logs/evidence.json` manifest together with coverage metrics and the thresholds they were measured against.

- Run the command once after the automated-test steps complete; verify exit code 0 and `RESULT: PASS` in the JSON outcome. A non-zero exit means a producer failed or capture failed — the evidence is still written, so the failure is evidenced, not hidden. Fix and re-run.
- The subcommand's child processes redirect the full global surface (`HOME`, `USERPROFILE`, `XDG_CONFIG_HOME`, `XDG_CACHE_HOME`, `APPDATA`, `LOCALAPPDATA`) so the developer's real global surface is never touched, and the run is hard-gated with a SHA-256 byte-identity fingerprint taken before and after.
- Evidence files are deterministic, machine-parseable, redacted of secrets, and within a size budget. The reviewer reads `openspec/changes/<name>/logs/` and cross-checks each producer entry against the `tasks.md` step it evidences, never silently assuming a step ran; missing or contradictory evidence for a mandatory step blocks the verdict.

## 9. Docs-Only Changes (NOT-APPLICABLE transformation)

For a change declaring `docs_only: true` in its `.nsoft.yaml` (the marker engages docs-only mode on its own, with no `skip_specs` gate), each mandatory automated-test-execution step above is marked NOT APPLICABLE with the justification that the diff touches no testable code, and steps labeled "AGENT MUST EXECUTE" become "AGENT MUST VERIFY NO-APPLICABILITY", performed through documental verification instead of test execution. The step names stay unchanged: the docs-only path replaces the producer set, it does not rename steps. The `collect` subcommand runs the `doc-checks` producer in place of this Application Type's test producers — it executes no test commands, performs documental checks of the change's documentation artifacts, and writes the deterministic report `logs/doc-checks.md` — and `logs/evidence.json` records `schemaVersion: 3` with `mode: "docs-only"` and no test-producer outcomes. The reviewer corroborates the docs-only change's claims from the `doc-checks` evidence and the documental evidence under `logs/agent/`; a failed `doc-checks` producer is a Blocker forcing `VERDICT: FAIL`, while the absence of test-run evidence for the NOT-APPLICABLE steps is a Warning that does not affect the verdict. A change may also declare `producers_not_applicable` in its `.nsoft.yaml`; each declared producer is recorded `NOT_APPLICABLE` with its non-empty justification in the `schemaVersion: 3` evidence and its command is not executed, and when both `docs_only: true` and `producers_not_applicable` are declared `docs_only` takes precedence and does not alter the docs-only producer map. A crate without the required `tests/e2e.rs` target declares the `e2e` producer not applicable with a justification. The safety-critical `scan` and `cli-smoke` producers are non-declarable: declaring either is a deterministic `INVALID_NOT_APPLICABLE_DECLARATION` capture failure.
