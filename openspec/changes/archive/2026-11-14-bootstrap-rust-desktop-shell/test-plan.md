---
schema: nsoft-spec-driven
artifact: test-plan
change: bootstrap-rust-desktop-shell
language: es
---

## Test Scope

Este plan de prueba define la estrategia de verificación del cambio `bootstrap-rust-desktop-shell` antes de la implementación. El cambio ejecuta las fases C1.2 a C1.5 del porte de escritorio StopWatch de Jira a Rust con destino a Gitea.

Está en alcance verificar:

- El shell de escritorio Rust con Iced 0.14.x: ventana principal vacía con título y tema, cierre limpio, bloqueo de instancia única, persistencia y restricción de geometría de ventana, suscriptor de logging y hook de pánico en el borde, y navegación de marcador de posición.
- Las puertas de calidad de CI: canal Linux y Windows con formato, clippy con advertencias como error, pruebas, cobertura y auditoría de dependencias; escaneo de SonarQube con línea base de cobertura del 90 %; prueba E2E en `tests/e2e.rs` contra directorio de datos de usuario aislado; captura de evidencia de `nsoft-specboot`; endurecimiento de CI con protección de rama `main`, Dependabot para Cargo y GitHub Actions, caché de compilación y concurrencia con cancel-in-progress.

Está fuera de alcance:

- Capacidades de la API de Gitea más allá de la autenticación con PAT y el modelo problema/comentario/tiempo de esta fase.
- Empaquetado y distribución de escritorio (fase posterior).
- Código fuente, pruebas de aplicación ni configuración: este artefacto es de planificación y no implementa nada.

## Approach

**Unit tests:** Pruebas unitarias sobre variantes de error tipadas con `thiserror` en `core`/`lib`, carga de configuración, restricción de geometría a monitores conectados, detección de archivo de bloqueo huérfano por proceso propietario, y rotación de logs basada en tamaño y tiempo con cantidad acotada de archivos retenidos.

**Integration tests:** Ejecución de las etapas del canal de CI (`cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test --all-targets`, `cargo audit`, `cargo deny check`) en ejecutores Linux y Windows, escaneo de SonarQube con el reporte de cobertura de `cargo llvm-cov`, y la prueba E2E de `tests/e2e.rs` contra un directorio de datos de usuario aislado con limpieza tras la ejecución.

**Smoke tests:** `cargo run` abre la ventana vacía de Iced y cierra limpiamente sin procesos residuales; un segundo lanzamiento enfoca la instancia existente y sale; la geometría persistida se restaura y se ajusta a los monitores conectados; el hook de pánico escribe en el archivo de log rotativo del directorio de datos de usuario sin incluir secretos.

**Manual testing:** Verificación de comportamiento en ejecutores Linux y Windows (Xvfb en Linux), comprobación de que las solicitudes a la API de Gitea autentican con el encabezado `Authorization: token` y se rechazan sin PAT válido, y confirmación de que la protección de rama `main` requiere las verificaciones de estado del CI antes de permitir la fusión.

## Coverage Targets

Los objetivos de cobertura se rastrean sobre los escenarios de las especificaciones:

- Spec: app-shell — Scenario: window-opens-with-title-and-theme
- Spec: app-shell — Scenario: clean-shutdown
- Spec: app-shell — Scenario: second-launch-focuses-existing-instance
- Spec: app-shell — Scenario: geometry-restored-and-clamped
- Spec: app-shell — Scenario: logging-and-panic-hook-wired
- Spec: app-shell — Scenario: placeholder-navigation-renders
- Spec: quality-gates — Scenario: ci-runs-on-linux-and-windows
- Spec: quality-gates — Scenario: sonarqube-produces-quality-gate-result
- Spec: quality-gates — Scenario: coverage-baseline-recorded
- Spec: quality-gates — Scenario: e2e-runs-against-isolated-user-data
- Spec: quality-gates — Scenario: evidence-manifest-produced
- Spec: quality-gates — Scenario: branch-protection-required
- Spec: quality-gates — Scenario: dependabot-opens-updates
- Spec: quality-gates — Scenario: ci-uses-cache-and-cancels
- Spec: quality-gates — Scenario: audit-and-deny-block-merge

**Coverage targets:** línea base del 90 % para líneas y funciones, medida con `cargo llvm-cov` y leída por SonarQube. Hasta que comience el trabajo de características, la cobertura se trata como tendencia, no como fallo duro de la puerta de CI.

## Success Criteria

El cambio se considera completo cuando se verifican todos los criterios siguientes:

- El crate compila y ejecuta una ventana vacía de Iced con título y tema definidos.
- `cargo fmt --check`, `cargo clippy -- -D warnings` y `cargo test --all-targets` pasan en Linux y Windows.
- La cobertura registrada es al menos del 90 % para líneas y funciones.
- `cargo audit` y `cargo deny check` pasan y bloquean la fusión cuando fallan.
- `tests/e2e.rs` existe, ejecuta contra un directorio de datos de usuario aislado y lo limpia al finalizar.
- El escaneo de SonarQube produce un resultado de puerta de calidad que bloquea la fusión cuando falla.
- La rama `main` requiere PR y verificaciones de estado del CI antes de la fusión.
- Dependabot abre pull requests de actualización para Cargo y GitHub Actions.
- El manifiesto de evidencia de `nsoft-specboot collect` se genera bajo `openspec/changes/bootstrap-rust-desktop-shell/logs/`.

## Notes

Preguntas abiertas heredadas del diseño: URL de la instancia de SonarQube e inyección de credenciales (variable de entorno vs almacén de secretos); tamaño exacto de rotación del log y cantidad de archivos retenidos; si el bloqueo de instancia única activa la instancia ejecutiva (traer al frente) o solo rechaza un segundo lanzamiento; formato de persistencia de geometría (JSON/TOML) y ubicación del archivo por plataforma.

Riesgos de verificación: el archivo de bloqueo puede quedar huérfano tras un fallo (mitigación: detección de proceso propietario vivo y eliminación de bloqueos huérfanos al inicio); CI en Windows puede diferir de Linux (bloqueo de archivos, separadores de ruta, backend de Iced) — se mitiga ejecutando CI en ambas plataformas y manteniendo abstracciones en `src/platform`; la disponibilidad de SonarQube se registra como SKIPPED con Advertencia en vez de FAIL si la instancia no es alcanzable.

Este artefacto es de planificación: no se crea, modifica ni elimina código fuente, pruebas ni configuración de aplicación.