---
description: This document contains all development rules and guidelines for this project, applicable to all AI agents.
alwaysApply: true
---

## 1. Core Principles

- **Small tasks, one at a time**: Always work in baby steps, one at a time. Never go forward more than one step.
- **Test-Driven Development**: Start with failing tests for any new functionality (TDD), according to the task details.
- **Type Safety**: All code must be fully typed.
- **Clear Naming**: Use clear, descriptive names for all variables and functions.
- **Incremental Changes**: Prefer incremental, focused changes over large, complex modifications.
- **Question Assumptions**: Always question assumptions and inferences.
- **Pattern Detection**: Detect and highlight repeated code patterns.

## 2. Language Standards
- **English Only**: All technical artifacts must always use English, including:
    - Code (variables, functions, classes, comments, error messages, log messages)
    - Documentation (README, guides, API docs)
    - Data schemas and database names
    - Configuration files and scripts
    - Git commit messages
    - Test names and descriptions

<!-- specboot:language-directive:start -->
Runtime-generated artifacts (the declared platform's issue titles/descriptions, the change-root `source-issue.md` record; OpenSpec prose: proposals, designs, test plans, tasks, reviews) follow the language recorded in the `language` key of `nsoft-project.properties` (`en` or `es`, default `en`).

Shipped documentation, code, Git artifacts, and machine-parsed tokens (file names, config keys, schema fields, commit message format) remain English-only unconditionally, regardless of the `language` value.
<!-- specboot:language-directive:end -->

### 2.1 Issue-Reference Hygiene

Real issue references (the declared platform's issue-reference format — a bare `#N` or an `owner/repo#N` cross-repository reference) are confined to `openspec/`. Outside `openspec/`, do not write a real reference: state the rationale as self-contained prose that stands on its own without the reference, and use the illustrative placeholder `JIRA-123` or `#123` for any example. Technical lookalike tokens such as `SHA-256`, `UTF-8`, and `AC-1` are not issue references and are exempt from this rule.

### 2.2 Issue and Project Model

This repository's issue and project model follows the declared forge platform and agrees with the `forge-issue-model` capability:
- **One issue per repository**: an issue belongs to exactly one repository, and work spanning several repositories is not represented by a single issue that spans them.
- **Owner-level Projects are the only cross-repository grouping**: issues are grouped above the repository level only through owner-level Projects; no other cross-repository grouping mechanism is used.
- **`type:`/`prio:`-prefixed labels carry type and priority**: an issue's type is carried by a `type:`-prefixed label and its priority by a `prio:`-prefixed label, and neither is carried by the issue state or the Project column.
- **Project column carries fine-grained status; open/closed is the issue state**: an issue's fine-grained workflow status is its Project card's column, while the issue's own state is limited to open or closed.
- **Automatic closure with manual board movement**: merging the pull request closes the issue through the closing keyword it carries, while moving the Project card between columns is a manual human step the tooling does not perform.
- **Labels consumed by name, a missing label is a no-op**: tooling references labels by name, and when a referenced label does not exist the operation is a no-op and the workflow continues.
- **The standard label set is provisioned outside the tooling**: the standard labels (including the `type:` and `prio:` set) are defined at the organization level on Gitea and at the repository level on GitHub; the tooling consumes them by name and never creates them, so a missing label is a no-op.
- **The hosting platform is declared per project**: the hosting platform is declared per project through the `forge` key (`gitea` or `github`).

## 3. Specific standards

For detailed standards and guidelines specific to different areas of the project, refer to:

<!-- specboot:standards-links:start -->
- [Desktop Standards](./standards-desktop.md) - Stack-neutral desktop process boundaries, typed inter-process contracts, local state, packaging and distribution, security posture, accessibility and testing expectations
- [Desktop Rust Standards](./standards-desktop-rust.md) - Rust toolchain and edition policy, crate layout, error-handling boundaries, the unsafe and FFI policy, framework mapping, the async runtime, supply-chain policy, the testing layers with the required end-to-end target, the 90 % coverage gate and the SonarQube posture
- [Documentation Standards](./standards-documentation.md) - Technical documentation structure, formatting, and maintenance guidelines
- [Deliverables Standards](./standards-deliverables.md) - Deliverable generation pipeline, placeholder tokens, revision rules, diagram assets, and bilingual en/es conventions
- [OpenSpec Tasks Mandatory Steps](./openspec-tasks-mandatory-steps.md) - Required checklist and execution rules when creating or updating OpenSpec `tasks.md` files
<!-- specboot:standards-links:end -->

## 4. Project Skills

- For consumers, skills live in the machine-global canonical root (under your home directory's config folder), with mirrors at `~/.config/opencode/skills/` and `~/.claude/skills/`; the project-local `.ai-specs/skills` copy exists only under `init --no-global`.
- When a request matches a skill, load and follow the corresponding `SKILL.md` automatically before continuing.
- Also load any referenced files in the skill folder (for example, `references/*.md`) when the skill requires them.

## 5. OpenSpec Tasks Mandatory Steps

When creating or updating OpenSpec `tasks.md` artifacts, follow these requirements.

### 5.1 Read OpenSpec configuration first

- **Required first action**: Read `openspec/config.yaml` before drafting or editing any `tasks.md` file.
- **Purpose**: Capture mandatory steps, naming conventions, task structure, testing expectations, and documentation requirements from project configuration.

### 5.2 Required task structure

For changes in this repository, ensure the checklist includes these mandatory steps in order:

<!-- specboot:mandatory-steps:start -->
1. **Step 0 (must be first)**: Create and switch to feature branch:
   - `feature/<issue-number>-<short-desc>` or `feature/<change-name>`
2. **Review and update existing unit tests (MANDATORY)**
3. **Run unit tests and coverage checks (MANDATORY)**
4. **End-to-end testing of the shipped desktop application (MANDATORY)** — AGENT MUST EXECUTE
5. **Update technical documentation (MANDATORY)**

For a change declaring `docs_only: true` in its `.nsoft.yaml` (the marker engages docs-only mode on its own, with no `skip_specs` gate), each mandatory automated-test-execution step above is marked NOT APPLICABLE with the justification that the diff touches no testable code, and steps labeled "AGENT MUST EXECUTE" become "AGENT MUST VERIFY NO-APPLICABILITY", performed through documental verification instead of test execution. The step names stay unchanged: the docs-only path replaces the producer set, it does not rename steps. A change may also declare `producers_not_applicable` in its `.nsoft.yaml`; each declared producer is recorded `NOT_APPLICABLE` with its non-empty justification and its command is not executed, and when both `docs_only: true` and `producers_not_applicable` are declared `docs_only` takes precedence and does not alter the docs-only producer map. A project whose crate carries no required `tests/e2e.rs` target declares the `e2e` producer not applicable with a justification, so the missing target is an explicit recorded decision rather than a failed capture. The safety-critical `scan` and `cli-smoke` producers are non-declarable: declaring either is a deterministic `INVALID_NOT_APPLICABLE_DECLARATION` capture failure. Evidence manifests declare `schemaVersion: 3`.
<!-- specboot:mandatory-steps:end -->

### 5.3 Agent-executed validation requirements

- **Never delegate required validation to the user** for steps required by `tasks.md`.
- The agent must execute all required automated and manual tests for its Application Type before marking tasks as completed (`[x]`). For a change declaring `docs_only: true` in its `.nsoft.yaml` (the marker engages docs-only mode on its own, with no `skip_specs` gate), the mandatory automated-test-execution steps are NOT APPLICABLE and the agent performs documental verification instead (see the §5.2 carve-out and `docs/openspec-tasks-mandatory-steps.md` §9).
- Generated output validation must confirm the expected file tree, symlink structure, and Windows/POSIX path expectations relevant to the change.
- **Isolate global scope, not only `cwd`**: Agent-executed CLI smoke tests must redirect the full global surface (`HOME`, `USERPROFILE`, `XDG_CONFIG_HOME`, `XDG_CACHE_HOME`, `APPDATA`, `LOCALAPPDATA`) into the test child process for the duration of the run — a scratch working directory alone cannot constrain home-relative and environment-resolved write targets — and must hard-gate the developer's real global surface with a SHA-256 byte-identity fingerprint taken before and after the runs.
- **Invocation timeout**: The capture step invokes `nsoft-specboot collect <change-name>` with an explicit environment/tool timeout parameter. The documented default is **300,000 ms (5 minutes)**, which exceeds the expected runtime of **60-90 seconds on standard hardware** with ~3x headroom. For slow CI machines, raise the timeout per run above the 5-minute default.
- Scratch directories and temporary test artifacts must be cleaned up before the validation step is considered complete.
- A producer validly declared `producers_not_applicable` in the change's `.nsoft.yaml` is recorded `NOT_APPLICABLE` with its non-empty justification in `logs/evidence.json`; its absent executed-command evidence is a valid state rather than a missing-evidence defect, and `docs_only: true` takes precedence over `producers_not_applicable` when both markers are declared. The safety-critical `scan` and `cli-smoke` producers are non-declarable: declaring either is a deterministic `INVALID_NOT_APPLICABLE_DECLARATION` capture failure.
- Agent-executed validation evidence SHALL be captured under `openspec/changes/<name>/logs/` via the committed capture helper before the step is marked complete, and is reviewer-verifiable from the change folder alone. For a docs-only change, `collect` runs the `doc-checks` producer instead of the Application Type's test producers and records `mode: "docs-only"` (schemaVersion 3) in `logs/evidence.json`.

### 5.4 Completion checklist before finalizing `tasks.md`

- Step 0 branch creation is first
- Mandatory steps are present and sequential
- Mandatory labels are explicit
- Branch name matches the repo convention
- Agent-executed manual testing tasks (when applicable to the Application Type) explicitly state "AGENT MUST EXECUTE"
- Generated output verification and cleanup steps are included
- Documentation update step is included

### 5.5 Mandatory artifact updates for change requests between `/apply` and `/archive`

If a new fix/change is requested after `/apply` and before `/archive`, treat it as a spec update first (never code-only first).

Required order:
1. Update affected OpenSpec artifacts (scenarios, requirements/specs, `tasks.md`).
2. Re-run artifact generation/update step when needed (`/continue`, `/ff`, or equivalent).
3. Implement code only after artifacts reflect the new request.
4. Re-run verification against updated artifacts before archiving.

Do not close this window with direct coding that is not reflected in OpenSpec artifacts.

## 6. Artifact Generation & Language Standards

Shipped documentation, code, Git artifacts, and machine-parsed tokens generated during any interaction remain English-only unconditionally, regardless of the language used in conversation. Runtime-generated prose — the declared forge platform's issue titles/descriptions, the change-root `source-issue.md` record, and OpenSpec prose (proposals, designs, test plans, tasks, reviews) — follows the `language` key recorded in `nsoft-project.properties`.

### 6.1 English-Only Requirement

- **Code artifacts**: Variables, functions, classes, constants, comments, error messages, log statements, and commit messages MUST be in English.
- **Documentation**: All technical specifications, guides, API documentation, and inline documentation MUST be in English.
- **Runtime-generated prose**: The declared forge platform's issue titles/descriptions, the change-root `source-issue.md` record, and OpenSpec prose (proposals, specifications, designs, test plans, tasks, and reviews) are runtime-generated prose and follow the `language` key recorded in `nsoft-project.properties`, while shipped documentation, code, Git artifacts, and machine-parsed tokens remain unconditionally English.
- **Markdown files**: Configuration documentation, standards, and guides MUST be in English.

### 6.2 Markdown Minimalism

Markdown files SHALL contain only line breaks that are strictly necessary according to punctuation and structural rules. Decorative or excessive spacing is prohibited.

- **Punctuation-driven breaks**: Line breaks appear only where required by punctuation (after sentences ending sections, before lists, between distinct concepts).
- **No decorative spacing**: Blank lines inserted purely for visual appeal are removed, preserving only functionally necessary breaks.
- **Natural flow**: Content maintains readability while respecting minimalism; unnecessary blank lines between related paragraphs are eliminated.

## 7. Explore Mode Principles

Explore mode is a conversational stance for deep thinking, not a structured workflow. The following principles clarify what is permitted and prohibited during exploration.

### 7.1 Explore Mode as Thinking Stance

Explore mode SHALL be a flexible, curiosity-driven conversation without mandatory steps or predetermined outcomes. Agents and users MAY follow any discussion path that emerges naturally.

- **No fixed workflow**: There are no required steps, no mandatory deliverables, and no prescribed sequence.
- **Adaptive exploration**: Agents follow interesting threads, pivot when new information emerges, and adapt without forcing a predetermined script.
- **Thinking time**: Exploration is valued as thinking time; the value is in the conversation itself, not in producing artifacts.

### 7.2 Permitted Exploration Activities

In explore mode, agents SHALL be allowed to:

- **Read and investigate**: Access files, search code, map architecture, and investigate codebase structure.
- **Ask clarifying questions**: Explore assumptions, challenge ideas, and reframe problems.
- **Analyze and visualize**: Create ASCII diagrams, build comparison tables, and sketch architectural concepts to clarify thinking.
- **Surface alternatives**: Present multiple options, trade-offs, and design approaches without prescribing a specific path.

### 7.3 Implementation is Prohibited

Agents SHALL NOT implement features, write application code, modify production files, or execute changes to the working codebase during exploration. Implementation is deferred to a separate phase after exploration concludes.

- **No code writing**: Application code, even in small increments, is not written during exploration.
- **No feature implementation**: Features are not built, tested, or deployed during exploration.
- **No file modification**: Implementation files are not edited, even if issues are identified.
- **Clear boundary**: When implementation is needed, agents remind users to exit explore mode and create a change proposal.

### 7.4 OpenSpec Artifact Creation is Permitted

Agents SHALL be able to create or update OpenSpec artifacts (proposals, specifications, designs, tasks) during exploration. These artifacts capture thinking, not implementation. All artifact creation follows English-only and Markdown minimalism standards.

- **Capture thinking**: Design decisions, requirement clarifications, and architectural insights MAY be captured in artifacts during exploration.
- **Offer before creating**: Agents offer to capture insights but do not auto-create files without user permission.
- **Maintain standards**: All OpenSpec artifacts generated in explore mode follow English-only and Markdown minimalism rules.

### 7.5 Exploration Guardrails

Agents SHALL maintain clear boundaries during exploration:

- **Never fake understanding**: If context is unclear, agents dig deeper rather than proceeding with incomplete information.
- **Never rush to conclusions**: Exploration takes time; agents do not force premature solutions or decisions.
- **Never force structure**: Patterns emerge naturally; agents do not impose rigid frameworks when flexible thinking serves better.
- **Never auto-capture**: Insights are offered for capture; agents do not create artifacts without explicit permission.

## 8. Git Commit Boundaries

File changes and git operations are independent actions. Commits occur only when explicitly triggered.

### 8.1 Independence of File Changes and Git Operations

File/content updates (task checkboxes in `tasks.md`, spec edits, code changes, documentation updates) and git operations (`git add`, `git commit`, `git push`, PR creation) are **independent actions**. Updating a work file is not a trigger for git operations.

### 8.2 Explicit Commit Triggers

A commit is triggered **only** by:
- An explicit user request (e.g., "commit this", "create a PR", "push these changes"), or
- Invocation of the `nssx-commit` skill (`.ai-specs/skills/nssx-commit/SKILL.md`)

In all other cases, agents MUST NOT run git operations.

**Sole exception**: the apply workflow's between-groups local commit (§8.4, §9.4) — a local commit delegated to `git-helper` between task-group delegations, with no push/PR/CI. This is apply-owned automation, not an agent-triggered commit.

### 8.3 Prohibitions for Agents

Agents MUST NOT:
- Run `git add`, `git commit`, `git push` as implicit side effects of completing tasks
- Run `git add`, `git commit`, `git push` as implicit side effects of updating `tasks.md` (e.g., marking a task `[x]`)
- Run `git add`, `git commit`, `git push` as implicit side effects of completing OpenSpec steps or implementation phases
- Create or update a PR without explicit user request or `nssx-commit` skill invocation

### 8.4 Authoritative Workflow for Git Operations

The `nssx-commit` skill (`.ai-specs/skills/nssx-commit/SKILL.md`) is the single authoritative workflow for all git write operations. All commits, pushes, and PR creation MUST use the `nssx-commit` skill workflow, never be done directly by agents without explicit user request.

**Exception — apply-owned between-groups local commit**: The apply workflow's between-groups commit step (verify → mark → commit → report, per §9.4 and the `apply-batch-width` capability) is the sole exception to this rule. The commit is delegated to the `git-helper` subagent, which creates a local `git add` + `git commit` of each completed task group's working-tree state, local only — no push, no pull request, and no CI trigger for any intermediate commit. The commit is a technical integrity anchor that protects each completed group's work from later reverts, not a human approval gate; automatic continuation is safe because each group is anchored by its own local commit. The full `nssx-commit` workflow (push, pull request creation, CI reporting) is reserved for the single later, user-triggered final step after apply finishes committing all groups, and is not part of apply's automated between-group behavior.

### 8.5 Correct Workflow Example

The correct workflow sequence is:

1. Implement change (write code, update specs, etc.)
2. Update work files (mark tasks `[x]` in `tasks.md`, update OpenSpec artifacts, etc.)
3. Stop — no commit yet
4. Only if the user explicitly asks, or the `nssx-commit` skill is invoked → run the `nssx-commit` skill workflow to stage, commit, push, and create/update a PR

### 8.6 Anti-Pattern

**Marking a task `[x]` in `tasks.md` is NOT a signal to commit.** File updates and git operations must remain separate. Commits are explicit only.

**Findings-resolution note**: When a review returns `VERDICT: FAIL`, the same local-commit-per-group mechanism applies: one `## N.` task group per Blocker or Major finding, one single group containing all Minor findings; each finding group's work is committed locally via `git-helper` delegation, following the verify → mark → commit → report cycle without stop. The `nsoft` orchestrator coordinates resolution following the same directives as the apply phase.

## 9. OpenSpec State Machine Compliance

This section defines how agents must interact with the OpenSpec state machine. It exists because speed-optimized models pre-plan aggressively and emit downstream artifacts — or implementation code — ahead of the user's continue signal, creating silent dependency gaps that no later check detects.

### 9.1 State is read from the CLI, never recalled from conversation

Before creating or revising any artifact, invoke `openspec status` to determine current change state. Re-read dependency artifacts from disk even when those files were already observed earlier in the same conversation — the user may have edited them between turns. Conversation memory is not a source of state.

### 9.2 Artifact status signals file existence, not correctness

An artifact reported as complete indicates only that its file exists. It does not indicate that the artifact was produced in dependency order or that the artifacts it derives from exist at all. Decide what still needs to be created from each artifact's declared dependency edges, not from its reported status.

### 9.3 The planning phase never writes implementation code

Skills that create or revise OpenSpec artifacts must not create, modify, or delete application source files, tests, or configuration. Implementation code is written only by the apply workflow, and only for tasks that already exist in the tasks artifact. When a revised plan implies code changes, stop and hand off to the apply workflow.

### 9.4 Batch width is owned by the invoked skill

The amount of work a single turn may produce is defined by the skill the user invoked. Honour that skill's guardrails exactly. Do not widen the batch on your own initiative regardless of confidence about what comes next. These widths are owned by the OpenSpec CLI and change between releases, so this protocol neither names skills, quotes their wording, nor enumerates their widths.

Apply's one-`## N.`-task-group-per-`developer`-delegation width is the first concrete instance of this protocol; its full protocol is defined by the `apply-batch-width` capability. Between groups apply runs the verify → mark → commit → report cycle (no human pause): it verifies the completed group, lets the `developer` mark its checkboxes, and delegates the commit to `git-helper`, which creates a local `git add` + `git commit` of the group's working-tree state before dispatching the next group's delegation. The between-groups commit is local only — no push, no pull request, no CI trigger for any intermediate commit — and a failed or blocked commit halts apply at that boundary exactly as a failed verification does. The commit is a technical integrity anchor that protects each completed group's work from later reverts, not a human approval gate; automatic continuation is safe because each group is anchored by its own local commit. The full `nssx-commit` workflow (push, pull request creation, CI reporting) is reserved for a single later user-triggered step after apply finishes committing all groups, and is not part of the automated between-group behavior.

**Findings-resolution note**: When a review returns `VERDICT: FAIL`, the same local-commit-per-group mechanism applies: one `## N.` task group per Blocker or Major finding, one single group containing all Minor findings; each finding group's work is committed locally via `git-helper` delegation, following the verify → mark → commit → report cycle without stop. The `nsoft` orchestrator coordinates resolution following the same directives as the apply phase.

### 9.5 Speculative pre-generation is prohibited

Do not draft, outline, or emit the content of an artifact that is not yet eligible for creation. Anticipating a later step's output in conversation is equivalent to writing the file early. Guidance describing a next step is guidance only and must never be acted upon within the same turn.

### 9.6 Rationale

These rules address verifiable failure modes: a silent dependency gap created when artifact status reports only file existence; late divergence surfacing at verification or in the review verdict; code written ahead of the tasks artifact being unreviewable against any specification; and batch widening that converts one fast turn into multiple unreviewable turns. The between-group commit is a technical integrity anchor that protects each completed group's work from later reverts, not a human approval gate; automatic continuation is safe because each group is anchored by its own local commit.

When a fix or change is requested after `/apply` and before `/archive`, the mandatory artifact update rule (§6.5) applies — treat it as a spec update first, never code-only first.

## 10. Project Context and Standards Gaps

`docs/project-context.md` is a descriptive record of the project's current state — its architecture, technology stack, domain conventions, and observed deviations from shipped standards. `docs/standards-gaps.md` records specific deviations or gaps between the normative standards and current project practice. The files in `docs/standards-*.md` are normative targets describing how the project SHOULD be built. Where `docs/project-context.md` or `docs/standards-gaps.md` describes an existing pattern that contradicts a rule in a `docs/standards-*.md` file, the normative standard takes precedence; an agent applying the rules follows the normative standard, not the conflicting descriptive project-context content. Both project-context and standards-gaps files carry sentinel frontmatter (`status: scaffold` or `status: customized`) to distinguish virgin scaffolds from hand-filled content.

## 11. OpenCode Managed Configuration Ownership

The managed OpenCode runtime block — the GLOBAL-classified keys `permission` (including `external_directory`), `mcp`, `model`, `small_model`, `enabled_providers`, `default_agent`, `compaction`, `watcher`, and the full `agent` roster — is written by the installer to the machine-scoped global configuration file `~/.config/opencode/opencode.jsonc` (shared across every project on the developer's machine), not to the project's root `opencode.json`. The `mcp` block (defining MCP servers and their credentials via `{env:VAR}` indirection) is managed and propagated by the installer as part of the packaged defaults. Consumers may customize entries via the named-entry merge mechanism: a customized entry is preserved, an absent entry is added from the package defaults, and a completely custom entry is never removed. Permission content — the root `permission` block and every `agent.<name>.permission` sub-key — is merged deep additively with consumer-wins at every leaf on both write paths (the machine-scoped global file and the `--no-global` project `opencode.json`): packaged sub-keys fill absent gaps, an existing consumer value at any leaf (scalar, array, or object, including type-mismatched leaves) stays verbatim, and consumer-added permission entries are never removed; because the merge never replaces a consumer value, it is never recorded as an overwrite. Every other field of an `agent` entry (`prompt`, `mode`, and any other) and all `mcp` entries keep full named-entry preservation; `agent.<name>.model` keeps named-entry preservation except with `init --force-agents`, and an existing agent entry lacking a `model` sub-key has the packaged default filled in; the remaining GLOBAL-classified keys (`model`, `small_model`, `enabled_providers`, `default_agent`, `compaction`, `watcher`) keep plain recursive overwrite. Deleting the global file restores the full packaged defaults on the next `init` (the reset path). The packaged `$schema` constant is unconditionally stamped into a freshly created global file; the residual `tools` key is installer-inert in global mode.

The packaged permission content keeps the V1 container (a `permission` map per agent and at the root) while carrying the action names the supported V2 client evaluates: `"shell": "allow"` is declared explicitly before each scoped `bash` map in the packaged roster, as authored contract that keeps exposure legible rather than inferred from a scope map; the supported client also normalizes a `bash` map into `shell` rules at load time, so that map both exposes the tool and scopes each command; and only a project-level `"shell": "deny"` removes the tool; `skill` must be granted explicitly because the deny-all key opens every map, and the V2 client normalizes the remaining V1 action names (`bash` → `shell`, `task` → `subagent`, `write`/`patch` → `edit`) at load time. This ownership model ensures deterministic MCP availability and agent-runtime defaults without requiring manual configuration steps in consumer projects.
