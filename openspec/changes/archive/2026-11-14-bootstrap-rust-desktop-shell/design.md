---
schema: nsoft-spec-driven
artifact: design
change: bootstrap-rust-desktop-shell
language: es
---

## Context

Proyecto de escritorio StopWatch que se migra de Jira a Rust con destino a Gitea (fases WBS 1-2). El descubrimiento, spikes y ADRs (C1.1) están completos; este cambio ejecuta C1.2 a C1.5.

Hechos de plataforma:
- Repositorio tipo `desktop`, stack `rust`, lenguaje de prosa OpenSpec `es` (`nsoft-project.properties`); el código, la documentación y los tokens de máquina siguen en inglés.
- Toolchain de Rust pinteada con `rust-toolchain.toml`; edición declarada en `Cargo.toml`.
- Framework de interfaz: Iced 0.14.x. Cliente HTTP: reqwest para la API de Gitea.
- Instancia Gitea: git.nsoft.com.ar (Gitea 28.3.0, OpenAPI 3.0.3). Los problemas se identifican como `owner/repo#N`; las entradas de tiempo se vinculan a comentarios de problema. Autenticación con PAT en el encabezado `Authorization: token`.
- Alcance del cambio: toolchain y layout `src/{main,lib,ui,core,platform}`, `forbid(unsafe_code)`/clippy/rustfmt, límites thiserror/anyhow, suscriptor tracing/hook de pánico/log rotativo, shell Iced (ventana vacía, bloqueo de instancia única, persistencia de geometría), CI (fmt/clippy/test/cobertura/audit-deny), SonarQube, `tests/e2e.rs`, protección de rama, Dependabot, `Swatinem/rust-cache`, concurrencia cancel-in-progress.
- Requisitos de comportamiento: `specs/app-shell/spec.md` y `specs/quality-gates/spec.md` (referenciados por nombre, no reescritos aquí). Motivación: `proposal.md` (leído desde disco).

Restricciones:
- Este artefacto es de planificación; no se escribe, modifica ni elimina código fuente, pruebas ni configuración de aplicación.
- Se escriben solo artefactos bajo `openspec/changes/bootstrap-rust-desktop-shell/`.
- Aplican `docs/standards-core.md` y `docs/standards-documentation.md`: código y documentación en inglés, minimalismo de Markdown, referencias a problemas confinadas a `openspec/`.
- Máquina de estados OpenSpec: el diseño es un artefacto de planificación que alimenta `test-plan` y `tasks`; no implementa.

## Goals / Non-Goals

**Goals:**
- Definir la arquitectura de implementación del shell de escritorio Rust y de las puertas de calidad de CI.
- Establecer como base para capacidades futuras el toolchain pinteado, el layout de crate, los límites de manejo de errores, la observabilidad (tracing/logs) y el shell de Iced.
- Configurar CI (fmt, clippy, test, cobertura, audit/deny), SonarQube, `tests/e2e.rs`, protección de rama, Dependabot, caché de compilación y concurrencia con cancel-in-progress.
- Servir de contrato técnico para `tasks.md` y para la fase de apply.

**Non-Goals:**
- No implementar código fuente, pruebas ni configuración de aplicación (esta es solo la fase de planificación).
- No definir requisitos funcionales (pertenece a `specs/app-shell/spec.md` y `specs/quality-gates/spec.md`).
- No cubrir capacidades de la API de Gitea más allá del autenticado con PAT y el modelo problema/comentario/tiempo de esta fase.
- No entregar empaquetado ni distribución de escritorio (fase posterior).

## Decisions

### D1. Toolchain pinning and crate layout

Decision: Pin el toolchain con `rust-toolchain.toml`, declarar la edición en `Cargo.toml`, y organizar el código bajo `src/{main,lib,ui,core,platform}` con `main` como punto de entrada del binario y `lib` exponiendo la API pública.
Alternatives considered:
- Toolchain sin pintear por el valor predeterminado de `rustup`: rechazado, rompe builds reproducibles entre CI y máquinas de desarrollador.
- `src/` plano con un solo `main.rs`: rechazado, no escala y mezcla responsabilidades.
- Espacio de trabajo con varios crates separados: rechazado, sobreingeniería para esta fase; un solo crate con layout de módulos es más simple y mantiene `cargo test --all-targets` significativo.
Rationale: Builds reproducibles, separación clara de responsabilidades (UI vs lógica de negocio vs abstracciones de plataforma), y alineación con el criterio de aceptación de que el crate compila y ejecuta una ventana vacía de Iced.

### D2. Error-handling boundaries

Decision: Usar `thiserror` para errores tipados de dominio en `core` y `lib`, y `anyhow` solo en el borde del binario (`main`) y en los arneses de prueba; nunca mezclar `anyhow` dentro del código de biblioteca.
Alternatives considered:
- `anyhow` en todas partes: rechazado, pierde información de error tipado a través de los límites de biblioteca.
- `thiserror` en todas partes incluyendo `main`: rechazado, `anyhow` es más ergonómico en el borde donde se añade contexto.
Rationale: Preserva errores tipados para los llamantes mientras mantiene el borde del binario ergonómico; cumple el requisito de límites thiserror/anyhow de la propuesta.

### D3. Observability stack

Decision: Suscriptor `tracing` para logging estructurado, hook de pánico que captura al log rotativo, y archivo de log rotativo con rotación por tamaño y tiempo.
Alternatives considered:
- `env_logger`: rechazado, menos estructurado y peor integración con código asíncrono de Iced/reqwest.
- Sin hook de pánico, confiar en el predeterminado de stderr: rechazado, una ventana de escritorio necesita diagnósticos de fallo persistidos.
Rationale: Logs estructurados y persistibles respaldan diagnósticos de una aplicación GUI donde una consola puede no ser visible.

### D4. Iced shell architecture

Decision: Ventana principal vacía con título y tema, bloqueo de instancia única mediante archivo de bloqueo en el directorio de datos de la plataforma, y persistencia de geometría (tamaño/posición) a un archivo de configuración; `core` posee la persistencia de configuración y `ui` posee la capa de vistas de Iced.
Alternatives considered:
- Mecanismos nativos de instancia única por SO (mutex/tubería con nombre): rechazado, un archivo de bloqueo es más simple, multiplataforma y suficiente para esta fase.
- Persistir geometría en cada frame: rechazado, escribir al débounce de redimensionar/mover o al cierre para evitar churn de I/O.
Rationale: Simplicidad multiplataforma y frecuencia de I/O segura; la ventana vacía cumple el criterio de aceptación de compilar y ejecutar.

### D5. CI pipeline

Decision: Pipeline Linux + Windows que ejecuta `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test --all-targets`, recolección de cobertura con línea base 90 %, `cargo audit` y `cargo deny check`, con `Swatinem/rust-cache` para caché de compilación y concurrencia cancel-in-progress en la rama por defecto.
Alternatives considered:
- Solo Linux: rechazado, la aplicación tiene como objetivo Windows también.
- Sin cancel-in-progress: rechazado, reduce el desperdicio de la cola de CI en PRs rebaseados.
Rationale: Cobertura en paralelo de ambas plataformas, retroalimentación rápida, gating de cadena de suministro y uso eficiente de caché.

### D6. Quality gates and testing

Decision: Escaneo de SonarQube como puerta de calidad, `tests/e2e.rs` ejecutado contra un directorio de datos de usuario aislado, línea base de cobertura del 90 % en líneas y funciones.
Alternatives considered:
- Sin SonarQube: rechazado, requerido por los criterios de aceptación y la postura de estándares.
- e2e contra datos de usuario reales: rechazado, el aislamiento evita mutar el estado de escritorio real del desarrollador.
Rationale: Cumple los criterios de aceptación y protege el entorno de desarrollo.

### D7. Repository governance

Decision: Protección de la rama `main` que requiere PR + verificaciones de estado, y Dependabot para Cargo y GitHub Actions.
Alternatives considered:
- Sin protección de rama: rechazado, contradice el flujo de merge controlado.
- Dependabot solo para Cargo: rechazado, los pins de GitHub Actions también necesitan actualización.
Rationale: Revisión forzada y frescura de dependencias.

## Risks / Trade-offs

[Risk] El archivo de bloqueo de instancia única puede quedar huérfano tras un fallo, bloqueando el relanzamiento. → Mitigation: Detección de bloqueo huérfano comprobando si el proceso propietario sigue vivo; eliminar bloqueos huérfanos al inicio.

[Risk] Los logs rotativos pueden crecer sin límite si la rotación está mal configurada. → Mitigation: Rotación basada en tamaño y tiempo con cantidad acotada de archivos retenidos; documentar la retención en `docs/project-context.md`.

[Risk] CI en Windows puede comportarse diferente de Linux (bloqueo de archivos, separadores de ruta, backend de Iced). → Mitigation: Ejecutar CI en ambas plataformas desde D1; mantener abstracciones de plataforma en `src/platform`; verificar con e2e en ejecutores de Windows cuando estén disponibles.

[Risk] Cobertura por debajo del 90 % bloquea la puerta mientras el shell es mínimo. → Mitigation: Línea base de cobertura en esta fase; añadir pruebas unitarias enfocadas en `core`/`ui` a medida que lleguen las características; tratar la cobertura como tendencia, no como fallo duro, hasta que comience el trabajo de características.

[Risk] `forbid(unsafe_code)` puede bloquear necesidades FFI futuras (p. ej., instancia única nativa o teclas rápidas globales). → Mitigation: Aislar cualquier FFI futuro detrás de `#[allow(unsafe_code)]` en `src/platform` con una razón de seguridad documentada, revisando el lint si fuera inevitable.

[Risk] Disponibilidad/credenciales de la instancia de SonarQube. → Mitigation: Configurar URL/token del escaneo desde variables de entorno; registrar el estado del escaneo como SKIPPED con una Advertencia en vez de FAIL si la instancia no es alcanzable, según la política de revisión.

## Migration Plan

1. Pintear el toolchain (`rust-toolchain.toml`) y establecer la edición en `Cargo.toml`; crear el layout `src/{main,lib,ui,core,platform}` con `forbid(unsafe_code)`, clippy y rustfmt configurados.
2. Implementar tipos de error con `thiserror` en `core`/`lib` y `anyhow` en el borde de `main`; conectar el suscriptor tracing, el hook de pánico y el log rotativo.
3. Construir el shell de Iced: ventana vacía con título/tema, bloqueo de instancia única y persistencia de geometría; verificar que compila y ejecuta.
4. Añadir flujos de trabajo de CI (Linux/Windows): fmt, clippy `-D warnings`, `cargo test --all-targets`, cobertura, `cargo audit`, `cargo deny check`; habilitar `Swatinem/rust-cache` y cancel-in-progress.
5. Configurar el escaneo de SonarQube y la puerta de calidad; crear `tests/e2e.rs` contra un directorio de datos de usuario aislado; registrar la línea base de cobertura del 90 %.
6. Habilitar la protección de la rama `main` y Dependabot para Cargo y GitHub Actions.
7. Verificar los criterios de aceptación: el crate compila y ejecuta una ventana vacía; `cargo fmt`, `cargo clippy -- -D warnings` y `cargo test --all-targets` pasan; cobertura en 90 %; `cargo audit` y `cargo deny check` pasan; `tests/e2e.rs` existe; el escaneo de SonarQube produce un resultado de puerta de calidad.
8. Actualizar `docs/project-context.md` y `docs/standards-gaps.md` para reflejar el nuevo stack y cualquier desviación.

## Open Questions

- ¿Cuál es la URL de la instancia de SonarQube y cómo se inyectan las credenciales (variable de entorno vs almacén de secretos)?
- Tamaño exacto de rotación del log y cantidad de archivos retenidos.
- Si el bloqueo de instancia única debe activar una instancia ya ejecutiva (traer al frente) o solo rechazar un segundo lanzamiento.
- Formato de persistencia de geometría (JSON/TOML) y ubicación del archivo por plataforma.
