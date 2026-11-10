## Review Scope

What aspects of the implementation were reviewed? Which modules, features, or scenarios were examined?

## Findings

Document any issues, observations, or concerns discovered during review:

- Code quality observations
- Test coverage gaps
- Performance or security concerns
- Documentation completeness
- Compliance with repository standards

## Verdict

Summarize the review outcome:

**VERDICT: PASS**

or

**VERDICT: FAIL**

---

**CRITICAL**: The VERDICT line MUST match exactly the pattern `^VERDICT:\s*(PASS|FAIL)\s*$`.

Examples of valid formats:
- `VERDICT: PASS`
- `VERDICT: FAIL`
- `VERDICT:     PASS` (with extra spaces)

Examples of invalid formats (will fail):
- `Verdict: PASS` (wrong case)
- `VERDICT:PASS` (no space after colon)
- `VERDICT : PASS` (space before colon)
- `verdict: pass` (wrong case)

The archiving process checks this line exactly. Ensure no variant spellings or case variations.

> **Docs-only variant (docs-lite)** — IF the change's `.nsoft.yaml` declares `docs_only: true` (the marker engages docs-only mode on its own, with no `skip_specs` gate), author the docs-lite variant: no SonarQube scan is required and no `## SonarQube Scan Evidence` section is recorded; record `QUALITY_GATE: SKIPPED (no code to scan)` as a Warning; derive the verdict from artifact coherence, cross-reference integrity, git-diff non-affectation of testable code, and docs-only marker legitimacy. The VERDICT line contract above applies unchanged.
>
> Use the standard `## SonarQube Scan Evidence` section below only when the change does NOT declare `docs_only: true`.

## SonarQube Scan Evidence

This section is mandatory. It backs the verdict line with the SonarQube scan that was run for this review; a review without this section, or with an empty or incomplete section, is an invalid review artifact regardless of its `VERDICT` line.

Required fields per evidence block:

- `SCAN_STATUS:` - `SUCCESS` / `FAILED` (absent when no scan ran)
- `QUALITY_GATE:` - `PASSED` / `FAILED` / `SKIPPED`
- `PROJECT_KEY:` - the SonarQube project key
- `ANALYSIS_TIMESTAMP:` - the analysis timestamp
- `EXIT_CODE:` - the scan exit code

Single application types record one block:

```
SCAN_STATUS: SUCCESS
QUALITY_GATE: PASSED
PROJECT_KEY: my-project
ANALYSIS_TIMESTAMP: 2026-01-01T00:00:00Z
EXIT_CODE: 0
```

Composite application types (`backend-frontend`, `backend-frontend-mobile`) record one block per runtime, each with its own five fields:

```
[backend]
SCAN_STATUS: SUCCESS
QUALITY_GATE: PASSED
PROJECT_KEY: my-project-backend
ANALYSIS_TIMESTAMP: 2026-01-01T00:00:00Z
EXIT_CODE: 0

[frontend]
SCAN_STATUS: FAILED
QUALITY_GATE: FAILED
PROJECT_KEY: my-project-frontend
ANALYSIS_TIMESTAMP: 2026-01-01T00:00:00Z
EXIT_CODE: 1
```

A scan that did not run, a missing or empty `SCAN_STATUS`/`QUALITY_GATE`, `SCAN_STATUS: FAILED`, or `QUALITY_GATE: FAILED` is a blocking cause for `VERDICT: FAIL`.

## Notes

Any additional context for the reviewer or for future reference.
