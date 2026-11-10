> **N/A producers** — IF a mandatory automated-test-execution step legitimately cannot run for this change, declare `producers_not_applicable` in the change's `.nsoft.yaml` as a mapping from the producer id to a non-empty justification; `collect` records each declared producer as `NOT_APPLICABLE` with that justification in the `schemaVersion: 3` evidence (`logs/evidence.json`) and does not execute its command. The nsoft-owned markers (`docs_only`, `producers_not_applicable`) live in `.nsoft.yaml`, never in `.openspec.yaml`; a `docs_only: true` marker engages docs-only mode on its own, with no `skip_specs` gate. When `docs_only: true` is also declared, `docs_only` takes precedence and the declaration does not alter the docs-only producer map. The safety-critical `scan` and `cli-smoke` producers are non-declarable: declaring either is a deterministic `INVALID_NOT_APPLICABLE_DECLARATION` capture failure.

## 1. <!-- Task Group Name -->

- [ ] 1.1 <!-- Task description -->
- [ ] 1.2 <!-- Task description -->

## 2. <!-- Task Group Name -->

- [ ] 2.1 <!-- Task description -->
- [ ] 2.2 <!-- Task description -->
