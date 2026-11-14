---
issue: null
status: draft
---

# Proposal

## Why

El cliente de escritorio StopWatch debe migrarse de Jira a Rust con destino a Gitea, cubriendo las fases WBS 1-2. El trabajo de descubrimiento, spikes y ADRs (C1.1) está completo; este cambio ejecuta C1.2 a C1.5: fundaciones de toolchain y layout, shell de Iced, CI/Sonar/E2E y endurecimiento de CI.

## What Changes

- Toolchain de Rust pinteada con `rust-toolchain.toml`, edition y perfil de release.
- Layout de crate `src/{main,lib,ui,core,platform}` con lints `forbid(unsafe_code)`, clippy y rustfmt.
- Errores tipados con `thiserror` y `anyhow` solo en el borde del binario; suscriptor `tracing`, hook de pánico y archivo de log rotativo.
- Shell de aplicación Iced: ventana principal vacía con título, tema, cierre limpio, bloqueo de instancia única y persistencia de geometría.
- Canal de CI para Linux y Windows: fmt, clippy, test, cobertura y audit/deny.
- Configuración y escaneo de SonarQube; `tests/e2e.rs` con directorio de datos de usuario aislado; cobertura base 90 %.
- Protección de rama `main`, Dependabot para Cargo y GitHub Actions, `Swatinem/rust-cache` y concurrencia con cancel-in-progress.

## Capabilities

### New

- `app-shell`: shell de escritorio Rust con Iced, ciclo de vida de ventana y persistencia de geometría.
- `quality-gates`: canal de CI, SonarQube, auditoría de dependencias, cobertura y E2E aislado.

### Modified

Ninguno; no hay requisitos de capacidad existentes que cambien.

## Acceptance Criteria

- Los ADRs de framework, identidad de issue, mapeo de tiempo/comentario y almacenamiento de credenciales están registrados.
- El crate compila y ejecuta una ventana vacía de Iced.
- `cargo fmt`, `cargo clippy -- -D warnings` y `cargo test --all-targets` pasan.
- Cobertura base registrada al 90 % para líneas y funciones.
- `cargo audit` y `cargo deny check` pasan.
- `tests/e2e.rs` existe y ejecuta contra un directorio de datos de usuario aislado.
- El escaneo de SonarQube produce un resultado de puerta de calidad.

Los criterios se toman del registro `source-issue.md` de issue #15.

## Impact

- `rust-toolchain.toml` y layout `src/{main,lib,ui,core,platform}`.
- Especificaciones nuevas: `specs/app-shell/spec.md` y `specs/quality-gates/spec.md`.
- Flujos de trabajo de CI, configuración de SonarQube y `tests/e2e.rs`.
- `docs/project-context.md` y `docs/standards-gaps.md` actualizados.
- Esta fase de planificación no modifica código fuente, pruebas ni configuración de aplicación.
