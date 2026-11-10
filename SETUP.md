# Project Setup After Installing @nsoft/specboot

You just ran `npx @nsoft/specboot init` in your project. `init` copied the manifest-resolved template files (`docs/`, `.ai-specs/`) for your selected Application Type, created the agent entrypoint symlinks (`AGENTS.md`, `CLAUDE.md`), wrote `nsoft-project.properties`, and updated `.gitignore`, `SETUP.md`, and `openspec/config.yaml`. The GLOBAL-classified OpenCode blocks (`permission`, `mcp`, `model`, `small_model`, `enabled_providers`, `default_agent`, `compaction`, `watcher`, `agent`) are written to the machine-scoped global config `~/.config/opencode/opencode.jsonc`, and the `nssx-*` skills are mirrored to the global mirror roots `~/.config/opencode/skills/` and `~/.claude/skills/`. See "Configuration Precedence" below for the global/project scope model.

`nsoft-project.properties` also records the project's declared forge (`gitea` or `github`), resolved from the `--forge` flag, an existing value, or an interactive prompt and defaulting to `gitea`. Every platform-touching skill and template reads that value and routes its platform operations to `git-helper` (Gitea) or `github-helper` (GitHub).

This guide covers the post-init setup steps for your project.

## Command Surface and Optional Plugins

The default `npm install @nsoft/specboot` installs five commands: `init`, `config`, `scan`, `collect` and `deliverable`. `init` is core's only built-in command; the other four come from four official command plugins that core declares as optional dependencies at its own exact version. "Optional" changes two things and nothing else: npm does not fail the install when one of those packages cannot be installed, and the opt-out below exists. The default command surface, and every command's name, flags, streams, exit codes and output bytes, are unchanged.

| Combination | Install | Command surface |
| --- | --- | --- |
| Default | `npm install @nsoft/specboot` | `init`, `config`, `scan`, `collect`, `deliverable` |
| Core only | `npm install @nsoft/specboot --omit=optional` | `init` |
| Core + one plugin | core-only install, then `npm install @nsoft/specboot-scan-plugin --omit=optional` | `init`, `scan` |
| Core + a subset | core-only install, then install exactly the plugins wanted with the same flag, for example `npm install @nsoft/specboot-scan-plugin @nsoft/specboot-collect-plugin --omit=optional` | `init`, `scan`, `collect` |

Three limits bound this matrix:

- **The opt-out is install-time and all-or-nothing.** npm offers no way to decline a single transitive optional dependency, so `--omit=optional` removes all four plugins together and there is no per-plugin removal switch; a narrower surface is the core-only install plus the plugins you explicitly install. The flag is also install-wide, so it omits the optional dependencies of every other package in your tree as well. The add-on install keeps the same flag: `--omit=optional` is install-wide, so a plugin install made with it cannot bring the other three plugins back.
- **A declaration cannot remove an official plugin.** `nsoft-project.properties#plugins` and `package.json#nsoftspecboot.plugins` only add ids — the official source is always appended — so no declaration removes an official command; only not installing the plugin does. An explicitly empty `plugins=` suppresses the `package.json` list and nothing else.
- **npm is the only supported package manager.** A layout that links packages from a store outside the project and core roots, such as a pnpm virtual store, is rejected when the plugin loads, so pnpm and Yarn layouts are unsupported.

**If you are upgrading, nothing disappears.** Upgrading an existing project keeps the four official command plugins installed: `npm install @nsoft/specboot@latest` reinstalls them as optional dependencies, and they remain installed on later `npm ci` runs. To switch to a core-only installation, run `npm install --omit=optional`; that flag is install-wide, so it also omits the optional dependencies of every other package in the tree, and a later plain install brings the plugins back while the lockfile still records them.

**Combinations that cannot work end to end** fail at the point of use, visibly and attributably, because no plugin learns whether another one is present:

- **`collect` without `scan`** — a producer set that declares the `scan` producer spawns `nsoft-specboot scan`; with that plugin absent the spawn is an unknown command, the producer is recorded `FAIL`, and `collect` exits non-zero.
- **`deliverable` without pandoc** — pandoc is a runtime prerequisite on `PATH`, not a package; generation stops before writing any file with `Error: pandoc was not found on PATH. ...` and exit `1`.
- **A workflow whose command is absent** — the installed templates and skills call `nsoft-specboot scan` and `nsoft-specboot deliverable`; in an installation without that plugin the step fails exactly as any unknown command does: the top-level usage on stderr, exit `1`, and no plugin diagnostic.

## Quick Start

### Prerequisites

`init` assumes you already installed and initialized OpenSpec in your project. If you have not:

```bash
npm install -g @fission-ai/openspec@latest
openspec init --profile custom
```

The supported coding-agent client is **OpenCode desktop V2** (verified against the managed `2.2.5` build); V1 runtimes are not supported. The packaged roster relies on V2 action semantics — `"shell": "allow"` declared explicitly before each scoped `bash` map as the authored contract that keeps exposure legible rather than inferred from a scope map, that same `bash` map normalized by the client into `shell` rules so it both exposes the tool and constrains each command, and the explicit `skill` grant — and since the V2 client ships no built-in `scout` subagent, the former `scout` entry is retired and pruned from an already-installed global configuration on the next `init` run.

### 1) Setup graphify

Graphify maps your entire project (code, docs, PDFs, images, videos) into a **knowledge graph** that your AI coding assistant can **query instead of grepping** through files. Set it up before onboarding so the codegraph substrate is in place.

[Install and setup instructions](https://github.com/Graphify-Labs/graphify)

### 2) Run `nssx-onboard` (Mandatory)

This step is required. If you skip it, your AI assistant will use generic technical context instead of your real project context.

`nssx-onboard` analyzes your brownfield codebase in a single invocation — covering stack, architecture patterns, conventions, domain language, API contracts, and data model. It both establishes the codegraph substrate (building `.codegraph/`) and writes the findings into `docs/project-context.md` and `docs/standards-gaps.md`, so any subsequent agent work can query the established code graph instead of relying on unaided inspection.

Invoke the skill with your copilot:

```text
/nssx-onboard
```

### 3) Point OpenSpec Config to Your `docs/` and `.ai-specs/`

After onboarding, update your project's `openspec/config.yaml` so its `context` references your technical context from `docs` and your agents from `.ai-specs/`. Prompt your copilot to do this for you:

```text
Update my openspec config.yaml context to reference this repository's docs and .ai-specs structure.

Requirements:
- Use docs/standards-core.md as the single source of truth.
- Include whichever `docs/standards-*.md` files were installed for your Application Type (for example `standards-backend.md`, `standards-frontend.md`, `standards-mobile.md`, or `standards-npm-package.md`), plus `docs/standards-documentation.md`.
- Include docs/api-spec.yml and docs/data-model.md when your Application Type has a backend track.
- Tell the agent to load the core agent from `.ai-specs/agents/developer.md`, which resolves the installed stack standards via the `specboot:standards-links` fragment in `docs/standards-core.md` §3.
- Mention .ai-specs/skills as workflow guidance.
- Keep all paths relative to the project root.
```

By the time you reach this step, `nssx-onboard` has already established the codegraph substrate and populated `docs/project-context.md` and `docs/standards-gaps.md`.

## Composite Project Layout

If you selected a composite Application Type (`backend-frontend` or `backend-frontend-mobile`), `init` created a `projects/` container folder at the repository root with each runtime nested inside it:

```
.
├── projects/
│   ├── backend/
│   │   ├── src/
│   │   ├── test/
│   │   └── sonar-project.properties
│   ├── frontend/
│   │   ├── src/
│   │   ├── test/
│   │   └── sonar-project.properties
│   └── mobile/            (backend-frontend-mobile only)
│       ├── src/
│       ├── test/
│       └── sonar-project.properties
└── nsoft-project.properties
```

Each runtime has its own `src/`, `test/`, and `sonar-project.properties` under `projects/<runtime>/`, with per-runtime SonarQube project keys (`<project>-backend`, `<project>-frontend`, `<project>-mobile`). The `nsoft-project.properties` file records `scanRoot.backend=projects/backend`, `scanRoot.frontend=projects/frontend`, and `scanRoot.mobile=projects/mobile` so tooling can locate each runtime.

**Migration for existing composite projects:** if you previously ran `init` with a composite Application Type and have runtime subdirectories at the repository root (for example `backend/`, `frontend/`), your next `init` run automatically relocates them into `projects/` via `git mv` and rewrites the `scanRoot.<runtime>` keys. The migration aborts if the working tree has uncommitted changes. To roll back, run `git mv projects/<runtime> <runtime>` for each runtime and restore the `scanRoot.<runtime>` keys in `nsoft-project.properties`.

## Verify Configuration

Your AI copilot should automatically load:

- **OpenCode**: `AGENTS.md` → `docs/standards-core.md`
- **Claude Code**: `CLAUDE.md` → `docs/standards-core.md`

All paths and rules are configured to work seamlessly without manual adjustments.

### Required MCP Environment Variables

Set the following environment variables to enable MCP server connectivity. Each variable credentials a specific MCP server:

- `OPENCODE_CONTEXT7_API_KEY` — context7 MCP server (documentation and context retrieval)
- `OPENCODE_TESTSPRITE_API_KEY` — testsprite MCP server (automated testing)
- `OPENCODE_SONARQUBE_TOKEN` — sonarqube MCP server (code quality scanning)
- `OPENCODE_SONARQUBE_URL` — sonarqube MCP server (code quality scanning)
- `OPENCODE_GITHUB_TOKEN` — github MCP server (GitHub API access)
- `OPENCODE_GITEA_TOKEN` — gitea MCP server (Gitea forge access). Obtain from your Gitea instance under **User Settings → Applications → Generate New Token** (`https://git.nsoft.com.ar/user/settings/applications`). Rotate by generating a new token and updating this variable.
- `OPENCODE_GITEA_HOST` — gitea MCP server (base URL of the Gitea instance to connect to). Default: `https://git.nsoft.com.ar`.

### Configuration Precedence

`init` writes the GLOBAL-classified OpenCode blocks (`permission`, `mcp`, `model`, `small_model`, `enabled_providers`, `default_agent`, `compaction`, `watcher`, `agent`) to a single machine-scoped global config file, `~/.config/opencode/opencode.jsonc`, shared across every project on the developer's machine — not to the project's root `opencode.json`. At runtime, OpenCode deep-merges the global config under any project-level OpenCode configuration; when the same setting exists in both, the project-level value takes precedence (project wins).

The `mcp` block and the `agent` roster are merged into the global file by named entry: an existing entry is preserved, an absent entry is added from the packaged defaults, and a completely custom entry is never removed. Permission content — the root `permission` block and every agent entry's `permission` sub-key — is merged deep additively with consumer-wins at every leaf: the packaged defaults fill only what is absent, every value you already set stays verbatim, and permission entries you added are never removed. Every other field of an agent entry and every existing `mcp` entry are left untouched; the remaining GLOBAL keys (`model`, `small_model`, `enabled_providers`, `default_agent`, `compaction`, `watcher`) are re-asserted by recursive overwrite, which replaces your customizations on every run. The same merge behavior applies to the project's own `opencode.json` under `init --no-global`. Deleting the global file restores the full packaged defaults on the next `init` run.

**Packaged permission posture:** every roster agent other than the prompt-less `util` opens its `permission` map with `"*": "deny"`, declares `"skill": "allow"` so any installed skill can be loaded, and declares `"shell": "allow"` immediately before its scoped `bash` map wherever it can run shell commands — the grant is the authored contract that keeps exposure legible, while the map, which the supported client normalizes into `shell` rules at load time, is what exposes the shell tool and decides whether an individual command executes; only a project-level `"shell": "deny"` leaves an agent that declares neither key without the tool. The `external_directory` allow-list is Linux-only (the POSIX `$HOME/...` entries plus `/tmp/opencode/*`), with no Windows drive-letter, backslash, or `AppData` entries, because Linux developer machines are the supported platform. `enabled_providers: ["opencode"]` is overwritten on every run as noted above, while provider choices you manage through `nsoft-specboot config` are written to the `.opencode/opencode.json` override layer, which wins at resolution time.

After a default `init` run, the relocation migration removes the project's root `opencode.json` and the project-local `.ai-specs/`, `.opencode/skills/`, and `.claude/skills/` directories — the canonical `.ai-specs/` source and the skill mirrors now live at the machine scope (`~/.config/nsoft-specboot/.ai-specs/`, `~/.config/opencode/skills/`, `~/.claude/skills/`).

**Hand-written comments in `~/.config/opencode/opencode.jsonc` are lost on merge.** `init` merges the file via a strip-parse-merge-restringify round-trip that does not preserve JSONC comments; the `init` summary warns naming the file when a commented global config was modified. This is an accepted trade-off.

Run `init --no-global` to skip writing the global configuration and skip the relocation migration together, keeping the full packaged defaults in the project's own `opencode.json`.

## Next Steps

After setup, follow the standard OpenSpec workflow with the `nssx-*` skills: enrich a story, start a change, generate the artifacts (`/opsx-continue` or `/opsx-ff`), apply tasks, verify, review, archive, and commit. See each `.ai-specs/skills/<name>/SKILL.md` for the full instructions.
