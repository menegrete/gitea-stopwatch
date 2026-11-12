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

- [ ] 4.1 Configurar canal de CI para Linux y Windows con `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test --all-targets` (espec: ci-pipeline)
- [ ] 4.2 Configurar cobertura con `cargo llvm-cov` y línea base del 90 % para líneas y funciones (espec: coverage-baseline)
- [ ] 4.3 Ejecutar `cargo audit` y `cargo deny check` en el canal de CI (espec: ci-pipeline, scenario audit-and-deny-block-merge)
- [ ] 4.4 Crear `tests/e2e.rs` con directorio de datos de usuario aislado y limpieza tras la ejecución (espec: e2e-test, scenario e2e-runs-against-isolated-user-data)
- [ ] 4.5 Configurar SonarQube para Rust y ejecutar el escaneo que produce el resultado de puerta de calidad (espec: sonarqube-scan)
- [ ] 4.6 Ejecutar `nsoft-specboot collect` y registrar el manifiesto de evidencia bajo `openspec/changes/bootstrap-rust-desktop-shell/logs/` (espec: evidence-capture)

## 5. Endurecimiento CI (C1.5)

- [ ] 5.1 Habilitar protección de la rama `main` con PR y verificaciones de estado requeridas (espec: ci-hardening, scenario branch-protection-required)
- [ ] 5.2 Añadir Dependabot para Cargo y GitHub Actions (espec: ci-hardening, scenario dependabot-opens-updates)
- [ ] 5.3 Añadir `Swatinem/rust-cache` y concurrencia con cancel-in-progress (espec: ci-hardening, scenario ci-uses-cache-and-cancels)
- [ ] 5.4 Ejecutar `cargo audit` y `cargo deny` en pull requests y con programación periódica (espec: ci-hardening, scenario audit-and-deny-block-merge)
- [ ] 5.5 Verificar que la puerta de calidad de SonarQube bloquea la fusión (espec: sonarqube-scan, scenario sonarqube-produces-quality-gate-result)

## 6. Validación y documentación (pasos obligatorios §5.2)

- [ ] 6.1 Revisar y actualizar las pruebas unitarias existentes (MANDATORIO)
- [ ] 6.2 Ejecutar pruebas unitarias y verificaciones de cobertura (MANDATORIO)
- [ ] 6.3 Pruebas de extremo a extremo de la aplicación de escritorio enviada (MANDATORIO — AGENTE DEBE EJECUTAR)
- [ ] 6.4 Actualizar la documentación técnica (MANDATORIO)
