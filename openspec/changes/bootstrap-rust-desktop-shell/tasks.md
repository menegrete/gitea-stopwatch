schema: nsoft-spec-driven
artifact: tasks
change: bootstrap-rust-desktop-shell
language: es

## 1. Paso 0 — Rama de funcionalidad (ya realizada)

- [x] 1.1 Crear y cambiar a la rama de funcionalidad `feature/15-bootstrap-rust-desktop-shell` (ya creada durante nssx-start)

## 2. Fundación Rust (C1.2)

- [x] 2.1 Pinear el toolchain con `rust-toolchain.toml` y declarar la edición en `Cargo.toml` (refs: specs/app-shell/spec.md, specs/quality-gates/spec.md, design D1)
- [x] 2.2 Crear el layout de crate `src/{main,lib,ui,core,platform}` con `main` como punto de entrada del binario y `lib` exponiendo la API pública (design D1)
- [x] 2.3 Configurar lints: `forbid(unsafe_code)`, clippy y rustfmt (design D1)
- [x] 2.4 Implementar tipos de error con `thiserror` en `core`/`lib` y `anyhow` solo en el borde de `main` (design D2)
- [x] 2.5 Conectar el suscriptor tracing, el hook de pánico y el log rotativo con rotación por tamaño y tiempo (design D3)
- [x] 2.6 Resolver directorios de datos de usuario y caché a través de la ubicación de plataforma (source-issue C1.2)

## 3. Shell Iced (C1.3)

- [x] 3.1 Crear ventana principal vacía con título y tema definidos, cierre limpio sin procesos residuales (espec: empty-main-window, scenarios window-opens-with-title-and-theme / clean-shutdown)
- [x] 3.2 Implementar bloqueo de instancia única mediante archivo de bloqueo en el directorio de datos de la plataforma (espec: single-instance-lock, scenario second-launch-focuses-existing-instance)
- [x] 3.3 Implementar persistencia de geometría (tamaño/posición) y ajuste a monitores conectados (espec: window-geometry-persistence, scenario geometry-restored-and-clamped)
- [x] 3.4 Renderizar navegación de marcador de posición para la futura ventana principal (espec: placeholder-navigation, scenario placeholder-navigation-renders)
- [x] 3.5 Verificar que `cargo run` abre la ventana vacía de Iced y cierra limpiamente (espec: logging-and-panic-hook, scenario logging-and-panic-hook-wired)

## 4. CI y calidad (C1.4)

- [x] 4.1 Configurar canal de CI para Linux y Windows con `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test --all-targets` (espec: ci-pipeline)
- [x] 4.2 Configurar cobertura con `cargo llvm-cov` y línea base del 90 % para líneas y funciones (espec: coverage-baseline)
- [x] 4.3 Ejecutar `cargo audit` y `cargo deny check` en el canal de CI (espec: ci-pipeline, scenario audit-and-deny-block-merge)
- [x] 4.4 Crear `tests/e2e.rs` con directorio de datos de usuario aislado y limpieza tras la ejecución (espec: e2e-test, scenario e2e-runs-against-isolated-user-data)
- [x] 4.5 Configurar SonarQube para Rust y ejecutar el escaneo que produce el resultado de puerta de calidad (espec: sonarqube-scan)
- [x] 4.6 Ejecutar `nsoft-specboot collect` y registrar el manifiesto de evidencia bajo `openspec/changes/bootstrap-rust-desktop-shell/logs/` (espec: evidence-capture)

## 5. Endurecimiento CI (C1.5)

- [x] 5.1 Habilitar protección de la rama `main` con PR y verificaciones de estado requeridas (espec: ci-hardening, scenario branch-protection-required)
- [x] 5.2 Añadir Dependabot para Cargo y GitHub Actions (espec: ci-hardening, scenario dependabot-opens-updates)
- [x] 5.3 Añadir `Swatinem/rust-cache` y concurrencia con cancel-in-progress (espec: ci-hardening, scenario ci-uses-cache-and-cancels)
- [x] 5.4 Ejecutar `cargo audit` y `cargo deny` en pull requests y con programación periódica (espec: ci-hardening, scenario audit-and-deny-block-merge)
- [x] 5.5 Verificar que la puerta de calidad de SonarQube bloquea la fusión (espec: sonarqube-scan, scenario sonarqube-produces-quality-gate-result)

## 6. Validación y documentación (pasos obligatorios §5.2)

- [x] 6.1 Revisar y actualizar las pruebas unitarias existentes (MANDATORIO)
- [x] 6.2 Ejecutar pruebas unitarias y verificaciones de cobertura (MANDATORIO)
- [x] 6.3 Pruebas de extremo a extremo de la aplicación de escritorio enviada (MANDATORIO — AGENTE DEBE EJECUTAR)
- [x] 6.4 Actualizar la documentación técnica (MANDATORIO)

## 7. Review Findings Resolution — Implementation Verification (Blocker)

- [x] 7.1 Verify committed implementation matches specs/app-shell/spec.md requirements (empty-main-window, single-instance-lock, window-geometry-persistence, logging-and-panic-hook, placeholder-navigation)
- [x] 7.2 Verify committed implementation matches specs/quality-gates/spec.md requirements (ci-pipeline, sonarqube-scan, coverage-baseline, e2e-test, evidence-capture, ci-hardening)

## 8. Review Findings Resolution — Coverage (Major)

- [x] 8.1 Add unit tests to improve coverage toward 90% threshold for core/ and platform/ modules
- [x] 8.2 Document coverage-as-trend rationale per design D6 until feature work begins

## 9. Review Findings Resolution — E2E Scope (Major)

- [x] 9.1 Expand tests/e2e.rs to validate instance lock (second launch focuses existing instance)
- [x] 9.2 Expand tests/e2e.rs to validate geometry persistence and clamp to monitors
- [x] 9.3 Expand tests/e2e.rs to validate logging-and-panic-hook writes without secrets

## 10. Review Findings Resolution — Warnings

- [x] 10.1 Document SonarQube quality gate configuration rationale (gate PASS with coverage trend)
- [x] 10.2 Improve evidence capture documentation for implementation verification

## 11. Blocker Resolution — Coverage Evidence Reconciliation

- [x] 11.1 Align the `coverage-baseline` spec requirement with the approved design D6 scoping (90% applies to unit-testable `core/`+`platform/`; GUI/binary edge excluded and covered by e2e), so spec, design, and evidence no longer contradict
- [x] 11.2 Reconcile `nsoft-specboot collect` coverage thresholds with that scope (or record thresholds matching the documented gate) so the evidence manifest is internally consistent
- [x] 11.3 Re-run `nsoft-specboot collect` and verify evidence.json records metrics consistent with the documented thresholds

## 12. Major Resolution — Single-Instance Focus

- [x] 12.1 Implement focusing the already-running instance on second launch (platform-specific: X11/Win32) or, if infeasible headless, implement a documented IPC/fallback that raises the existing window; cover with a unit/integration test where possible

## 13. Major Resolution — tracing Facade

- [x] 13.1 Adopt the `tracing` facade crate (v0.1.44 available) and route the rolling-file writer + panic hook through a `tracing` subscriber layer, satisfying standard/D3/task 2.5 without requiring the unavailable tracing-subscriber 0.2 stack

## 14. Major Resolution — CI Coverage Stage

- [x] 14.1 Add the coverage stage (`cargo llvm-cov --lcov`) to `.github/workflows/ci.yml`, enforcing the documented coverage scope/threshold consistent with group 11

## 15. Major Resolution — E2E Drives Shipped Binary

- [x] 15.1 Add an e2e test that launches the shipped binary (headless via Xvfb where available, or a documented harness) and asserts window open / clean shutdown / title / theme / placeholder navigation are exercised by execution

## 16. Major Resolution — Cross-Platform Orphan-Lock Recovery

- [x] 16.1 Implement cross-platform process-liveness check for orphan-lock recovery (`is_process_alive` must not assume alive on non-Linux: Linux and Windows both resolve it through the safe `sysinfo` process table) and make the liveness tests platform-neutral so `cargo test --all-targets` passes on both CI operating systems

## 17. Major Resolution — Geometry Clamp Wiring

- [x] 17.1 Wire real monitor enumeration into the runtime boot path so geometry is clamped to connected monitors (replace `clamp_to_monitor(geometry, None)` with the detected monitor bounds); test the wiring logic

## 18. Minor Resolution — Toolchain, Release Profile, Lints

- [x] 18.1 Pin the toolchain to a specific version in `rust-toolchain.toml` and add a `[profile.release]` section to `Cargo.toml`
- [x] 18.2 Populate `clippy.toml`/`rustfmt.toml` and add crate-root lint denies in `src/lib.rs`/`src/main.rs`
- [x] 18.3 Document the two Warnings (branch-protection server-side, Playwright SKIPPED) in `docs/standards-gaps.md`

## 19. Blocker Resolution — SonarQube Scan Coverage Linking

- [x] 19.1 Diagnose and fix the `cargo llvm-cov` / `nsoft-specboot scan` link failure when instrumenting `tests/e2e.rs` (rust-lld `debug_str` offset / `R_X86_64_32 out of range`) — e.g. reduce instrumented debug info via `[profile.dev] debug = 1` / `[profile.test] debug = 1` (`line-tables-only`) and disable incremental for instrumented builds
- [x] 19.2 Verify `nsoft-specboot scan` at HEAD exits with a fresh analysis (scanStatus SUCCESS; qualityGate PASSED or SKIPPED) and align the CI `Run coverage` stage with the working configuration

## 20. Major Resolution — Remove Unsafe Env Mutation in E2E

- [ ] 20.1 Refactor `tests/e2e.rs` env-mutating helpers (`override_env`/`restore_env`) to explicit path seams so no `unsafe { env::set_var }` remains and parallel tests no longer race on process env

## 21. Major Resolution — Commit Planning Artifacts

- [ ] 21.1 Track the change's planning artifacts that are still untracked (`proposal.md`, `design.md`, `test-plan.md`, `specs/app-shell/spec.md`, `source-issue.md`, `.openspec.yaml`) so the normative sources are in git

## 22. Minor Resolution — Review Minors

- [ ] 22.1 Fix stale documentation numbers (test counts, coverage figures) in docs
- [ ] 22.2 Report coverage `branches` as null / not-applicable when the denominator is 0 instead of a misleading 100
- [ ] 22.3 Bound the orphan-lock recovery recursion (convert to a loop or add a retry cap)
- [ ] 22.4 Make the `LLVM_COV_FLAGS` scope regex portable/document the platform difference
- [ ] 22.5 Add `--locked` to `cargo install` steps in CI
- [ ] 22.6 Fix malformed `tasks.md` frontmatter (proper `---` fences) and verify `openspec status --change bootstrap-rust-desktop-shell --json` still parses
