## Purpose
Las puertas de calidad automatizan la verificación del proyecto Rust: canal de CI para Linux y Windows con formato, clippy, pruebas, cobertura y auditoría de dependencias, configuración y escaneo de SonarQube, prueba E2E con directorio de datos de usuario aislado, cobertura base del 90 %, captura de evidencia de nsoft-specboot, y endurecimiento de CI con protección de rama, Dependabot, caché y concurrencia.

## ADDED Requirements

### Requirement: ci-pipeline
El canal de CI SHALL ejecutar en Linux y Windows: formato, clippy con advertencias como error, pruebas, cobertura y auditoría de dependencias.

#### Scenario: ci-runs-on-linux-and-windows
- **WHEN** se abre un pull request o se envía un cambio a la rama principal
- **THEN** el canal de CI se ejecuta en Linux y Windows aplicando formato, clippy, pruebas, cobertura y auditoría de dependencias

### Requirement: sonarqube-scan
El proyecto SHALL configurar SonarQube para Rust y SHALL ejecutar el escaneo que produce un resultado de puerta de calidad.

#### Scenario: sonarqube-produces-quality-gate-result
- **WHEN** se ejecuta el escaneo de SonarQube con el reporte de cobertura de cargo llvm-cov
- **THEN** se produce un resultado de puerta de calidad que bloquea la fusión cuando falla

### Requirement: coverage-baseline
La línea base de cobertura del 90 % en líneas y funciones SHALL aplicarse al alcance verificable con pruebas unitarias: `src/core/` y `src/platform/`. La cobertura global del proyecto SHALL registrarse como tendencia hasta que comience el trabajo de características.

#### Scenario: coverage-baseline-recorded
- **WHEN** se mide la cobertura del alcance verificable con pruebas unitarias mediante `cargo llvm-cov --lcov`
- **THEN** la cobertura registrada para `src/core/` y `src/platform/` es al menos del 90 % en líneas y funciones, y el manifiesto de evidencia la registra frente a ese umbral del 90 %

#### Scenario: coverage-excludes-display-bound-code
- **WHEN** se mide la cobertura del alcance verificable con pruebas unitarias
- **THEN** la capa GUI de Iced (`src/ui/`), el bucle de aplicación (`src/lib.rs`) y el borde binario (`src/main.rs`) quedan excluidos por la configuración de medición, sin reducir el umbral del 90 %

#### Scenario: coverage-of-display-bound-code-verified-by-e2e
- **WHEN** se verifica el comportamiento de la capa GUI de Iced, del bucle de aplicación y del borde binario
- **THEN** esa verificación se realiza mediante la prueba de extremo a extremo y no mediante la línea base de cobertura unitaria

### Requirement: e2e-test
Las pruebas de extremo a extremo SHALL existir en tests/e2e.rs y SHALL ejecutarse contra un directorio de datos de usuario aislado, limpiando después de su ejecución.

#### Scenario: e2e-runs-against-isolated-user-data
- **WHEN** se ejecuta tests/e2e.rs
- **THEN** la prueba usa un directorio de datos de usuario aislado y lo limpia al finalizar

### Requirement: evidence-capture
La captura de evidencia de nsoft-specboot SHALL producirse para el cambio y registrarse bajo openspec/changes/<change>/logs/.

#### Scenario: evidence-manifest-produced
- **WHEN** se ejecuta nsoft-specboot collect para el cambio
- **THEN** se genera un manifiesto de evidencia bajo openspec/changes/<change>/logs/

### Requirement: ci-hardening
La rama principal SHALL tener protección con verificaciones de estado requeridas, Dependabot para Cargo y GitHub Actions, caché de Rust y concurrencia con cancelación en progreso.

#### Scenario: branch-protection-required
- **WHEN** se intenta fusionar en la rama principal
- **THEN** se requieren las verificaciones de estado del CI antes de permitir la fusión

#### Scenario: dependabot-opens-updates
- **WHEN** hay actualizaciones disponibles para Cargo o GitHub Actions
- **THEN** Dependabot abre pull requests de actualización

#### Scenario: ci-uses-cache-and-cancels
- **WHEN** se ejecuta una canalización de CI
- **THEN** se usa el caché de Rust y se cancelan las ejecuciones supersedidas

#### Scenario: audit-and-deny-block-merge
- **WHEN** cargo audit o cargo deny falla
- **THEN** la fusión queda bloqueada
