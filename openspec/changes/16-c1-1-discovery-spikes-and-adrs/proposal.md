---
issue: null
status: draft
---

## Why

El puerto del cronómetro al stack de escritorio aún no tiene resueltas las incógnitas técnicas que condicionan el código: autenticación con la API de Gitea, capacidades del framework Iced (ventanas múltiples, accesibilidad, empaquetado), almacenamiento de credenciales y la identificación de issues. Este cambio resuelve esos desconocidos con spikes y ADR antes de escribir código y congela el alcance del puerto mediante una matriz de paridad.

## What Changes

- Spike de la API de Gitea: autenticación PAT, GET /user, obtención de issues, POST /times, POST /comments, mapeo de errores y unidad timetracking.
- Spike de Iced: ventanas múltiples, siempre arriba, arrastre y redimensionado, accesibilidad, iced_test y empaquetado.
- ADRs: framework, identidad de issue (owner/repo#N), mapeo tiempo/comentario y almacenamiento de credenciales.
- Matriz de paridad: qué se porta, adapta o descarta.
- Actualización de docs/project-context.md y docs/standards-gaps.md.

## Capabilities

### New Capabilities

- gitea-api: cliente de la API de Gitea (auth PAT, issues, times, comments, mapeo de errores, timetracking).
- iced-desktop: shell de escritorio basado en Iced (ventanas múltiples, siempre arriba, arrastre/redimensionado, accesibilidad, empaquetado).
- adr-registro: decisiones de arquitectura para framework, identidad de issue, mapeo tiempo/comentario y credenciales.
- matriz-paridad: alcance del puerto (portado, adaptado, descartado).

### Modified Capabilities

No hay especificaciones existentes bajo openspec/specs/, por tanto no se listan capacidades modificadas.

## Acceptance Criteria

- Notas de spike registradas, incluidos límites de API y brechas del framework.
- ADR aprobados.
- Matriz de paridad congelada.
- docs/project-context.md y docs/standards-gaps.md actualizados.

## Impact

Código nuevo bajo specs/ (uno por capacidad), documentación técnica (project-context.md, standards-gaps.md) y futuras implementaciones bajo src/; sin cambios de API públicos ni dependencias nuevas aún definidas.
