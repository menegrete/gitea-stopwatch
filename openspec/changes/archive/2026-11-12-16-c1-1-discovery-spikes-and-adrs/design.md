## Context

Estado actual: el puerto del cronómetro al stack de escritorio tiene incógnitas técnicas sin resolver que condicionan cualquier implementación: autenticación con la API de Gitea, capacidades del framework Iced (ventanas múltiples, accesibilidad, empaquetado), almacenamiento de credenciales y la identidad de los issues. No existen especificaciones previas bajo openspec/specs/. Véase proposal.md - Why para la motivación; no se repite aquí.

Restricciones: la prosa de este documento sigue language=es de nsoft-project.properties; los tokens máquina (nombres de archivo, rutas, claves de config, formatos de commit) permanecen en inglés. Se aplican docs/standards-core.md y docs/standards-documentation.md.

## Goals / Non-Goals

**Goals:**
- Definir el enfoque para resolver las incógnitas técnicas mediante spikes y ADR antes de escribir código de implementación.
- Establecer la matriz de paridad que clasifica cada capacidad como portada, adaptada o descartada y congela el alcance del puerto.
- Producir los cuatro artefactos bajo specs/ (gitea-api, iced-desktop, adr-registro, matriz-paridad) y actualizar docs/project-context.md y docs/standards-gaps.md con los hallazgos.

**Non-Goals:**
- Escribir código de implementación bajo src/.
- Definir contratos de API definitivos más allá de los hallazgos de los spikes.
- Empaquetar o distribuir la aplicación de escritorio.

## Decisions

**D1 — Estructura de los spikes.** Los spikes se ejecutan como investigaciones acotadas en el tiempo con notas registradas (límites de API, brechas del framework) en lugar de código de producción. Alternativa: escribir pruebas de concepto directamente como código de producción. Razón: las notas de spike son un criterio de aceptación del cambio y deben permanecer como aprendizajes desechables, no como implementación prematurea.

**D2 — ADR como documentos independientes.** Cada decisión de arquitectura se almacena como un documento independiente con título, contexto, decisión, consecuencias y fecha, con identificador único e historial consultable (espec adr-registro). Alternativa: un único archivo consolidado de ADR. Razón: los documentos independientes permiten consultar el historial completo y asignar identificadores únicos por decisión.

**D3 — Identidad de issue como owner/repo#N.** Cada issue se identifica de forma inequívoca con el formato owner/repo#N; los identificadores que no cumplan el formato se rechazan (espec adr-registro). Alternativa: número de issue simple. Razón: el modelo del forge establece un issue por repositorio, por lo tanto la identidad debe ser inequívoca a través de repositorios.

**D4 — Almacenamiento de credenciales cifrado.** Las credenciales de la API de Gitea se almacenan cifradas en el almacén de credenciales del sistema operativo, se recuperan solo para el usuario autenticado y nunca se exponen en texto plano (espec adr-registro). Alternativa: archivo de configuración en texto plano. Razón: la postura de seguridad de escritorio exige protección de credenciales en reposo.

**D5 — Cliente de API de Gitea.** El cliente autentica cada solicitud con un PAT en el encabezado Authorization, rechaza solicitudes sin PAT válido, mapea códigos HTTP a errores de dominio (autenticación, cliente, servidor con reintento) y expone una unidad de control de tiempo por issue (espec gitea-api). Las decisiones de librería HTTP y runtime se dejan al spike.

**D6 — Matriz de paridad y alcance congelado.** Cada capacidad se clasifica como portada, adaptada o descartada con nombre, clasificación y justificación, con trazabilidad al spike o ADR que fundamenta la decisión (espec matriz-paridad). El alcance congelado solo cambia con aprobación documentada. Alternativa: alcance abierto sin congelamiento. Razón: el cambio congela el alcance del puerto para evitar expansión no controlada.

## Risks / Trade-offs

[Riesgo] El framework Iced puede no soportar ventanas múltiples de forma estable en la versión objetivo. → Mitigación: el spike de iced-desktop valida ventanas múltiples antes de comprometerse; fallback a ventana única si no es viable.

[Riesgo] El almacén de credenciales difiere por plataforma (macOS Keychain, Windows Credential Manager, Linux Secret Service). → Mitigación: abstraer el almacén detrás de una interfaz con implementaciones por plataforma, decididas en el spike y el ADR de credenciales.

[Riesgo] Límites de tasa de la API de Gitea afectan al registro de tiempo y comentarios. → Mitigación: el spike de gitea-api registra los límites y el cliente implementa reintento para errores de servidor (500).

[Riesgo] El alcance congelado de la matriz de paridad dificulta la iteración posterior. → Mitigación: los cambios requieren aprobación documentada según el espec, lo que controla la expansión sin bloquear completamente la evolución.

## Migration Plan

Pasos para desplegar:
1. Ejecutar el spike de gitea-api: autenticación PAT, GET /user, obtención de issues, POST /times, POST /comments, mapeo de errores y unidad timetracking; registrar notas incl. límites de API.
2. Ejecutar el spike de iced-desktop: ventanas múltiples, siempre arriba, arrastre y redimensionado, accesibilidad, iced_test headless y empaquetado; registrar brechas del framework.
3. Redactar y aprobar los ADRs: framework, identidad de issue (owner/repo#N), mapeo tiempo/comentario y almacenamiento de credenciales.
4. Congelar la matriz de paridad: clasificar cada capacidad con justificación y trazabilidad al spike o ADR correspondiente.
5. Actualizar docs/project-context.md y docs/standards-gaps.md con los hallazgos de los spikes y las decisiones aprobadas.

Estrategia de rollback: los spikes y los ADR son artefactos de descubrimiento sin código productivo, por lo que el rollback consiste en descartar las notas de spike y revertir los ADR y la matriz a la versión previa; no hay migración de datos ni cambios de API pública que revertir.

## Open Questions

- ¿Qué crate HTTP se usa para el cliente de la API de Gitea (reqwest u otra)? Se resuelve en el spike de gitea-api y se documenta en el ADR de framework.
- ¿Iced soporta ventanas múltiples de forma estable en la versión objetivo? Se resuelve en el spike de iced-desktop.
- ¿Qué almacén de credenciales por plataforma se usa? Se resuelve en el spike y el ADR de almacenamiento de credenciales.

Estas preguntas son genuinamente aplazables: los spikes están diseñados precisamente para responderlas, por lo que no modifican los specs, el enfoque ni las tareas del cambio.
