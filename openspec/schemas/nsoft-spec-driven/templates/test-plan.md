> **Docs-only variant (LITE)** — IF the change's `.nsoft.yaml` declares `docs_only: true` (the marker engages docs-only mode on its own, with no `skip_specs` gate), author the LITE documental checklist below instead of the standard sections. The `test-plan.md` artifact is still produced, so the six-artifact dependency chain stays intact.
>
> - **Test Scope**: scope is the change's documentation artifacts — their structure, content, and cross-references — plus an explicit record that the diff touches no testable code (no `.ts`, `.cs`, `.tsx`, or `.js` source file, pipeline script, or CI configuration that executes tests).
> - **Approach**: documental verification only. Unit, integration, and smoke testing and spec-scenario coverage targets are OUT OF SCOPE for a docs-only change; do not plan or promise test execution.
> - **Coverage Targets**: cross-reference the docs-only scenarios from the specs that documental verification must confirm (e.g., "Spec: docs-only-workflow — Scenario: Docs-only test-plan is documental").
> - **Success Criteria**: the change's documentation artifacts are structurally complete, coherent, and correctly cross-referenced, and the diff touches no testable code.
> - **Notes**: any documental assumptions relevant to verifying this change.

For a change that does NOT declare `docs_only: true`, use the standard structure below.

## Test Scope

Define the scope of verification: what subsystems, features, or requirements are subject to testing? What is explicitly out of scope?

## Approach

Describe the testing methodology:

- **Unit tests**: Which modules or functions require unit-level coverage?
- **Integration tests**: Which component interactions must be verified?
- **Smoke tests**: Quick end-to-end validation steps
- **Manual testing**: Any steps that cannot be automated
- **Coverage targets**: Reference specific scenarios from specs.md that must pass

## Coverage Targets

List the spec scenarios that this test plan will verify. Cross-reference each scenario from the specs (e.g., "Spec: user-auth — Scenario: Successful login") with the test approach that covers it.

## Success Criteria

How do you know testing is complete and passed?

- All unit tests pass
- All integration tests pass
- Specified manual tests show expected behavior
- No regressions in existing tests
- Code coverage meets project standards (if applicable)

## Notes

Any other testing notes, dependencies, or assumptions relevant to verifying this change.
