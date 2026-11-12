## Test Scope

Este plan de verificación cubre los artefactos de descubrimiento del cambio 16-c1-1-discovery-spikes-and-adrs: las cuatro especificaciones bajo specs/ (gitea-api, iced-desktop, adr-registro, matriz-paridad), las notas de spike, los ADR aprobados, la matriz de paridad congelada y las actualizaciones a docs/project-context.md y docs/standards-gaps.md. El alcance de verificación es estructural y de contenido: confirmar que cada escenario de cada especificación está cubierto por el correspondiente artefacto de descubrimiento, que los ADR contienen título, contexto, decisión, consecuencias y fecha con identificador único, y que la matriz de paridad clasifica cada capacidad con justificación y trazabilidad. Quedan explícitamente fuera de scope la ejecución de pruebas unitarias, de integración o de humo sobre código de producto, pues este cambio no entrega implementación bajo src/ ni dependencias ejecutables.

## Approach

La verificación se ejecuta antes de cualquier implementación y se fundamenta en revisión documental y conformidad con las especificaciones.

- **Pruebas unitarias**: No aplican. El cambio no entrega código de producto bajo src/ ni módulos con lógica ejecutable susceptible de cobertura unitaria.
- **Pruebas de integración**: No aplican. No hay interacciones de componentes implementados que verificar; las interacciones descritas (cliente API de Gitea, shell Iced, almacén de credenciales) se validan indirectamente mediante las notas de spike y los ADR.
- **Pruebas de humo**: No aplican. No existe binario ni aplicación empaquetada que ejecutar en este cambio.
- **Pruebas manuales**: Revisión de cada documento de descubrimiento contra su especificación: completitud de escenarios, coherencia de decisiones entre ADR y matriz de paridad, trazabilidad de clasificaciones, y correcta actualización de project-context.md y standards-gaps.md.
- **Cobertura de objetivos**: Cada escenario de las cuatro especificaciones se cross-referencea en la sección Cobertura de Objetivos y se verifica mediante revisión documental del artefacto correspondiente.

## Coverage Targets

- Spec: gitea-api — Scenario: Solicitud autenticada con PAT válido → verificado por revisión de la nota de spike de gitea-api y el ADR de autenticación.
- Spec: gitea-api — Scenario: Solicitud sin PAT configurado → verificado por revisión de la nota de spike y el ADR de almacenamiento de credenciales.
- Spec: gitea-api — Scenario: Consulta exitosa del usuario autenticado → verificado por la nota de spike de gitea-api (GET /user).
- Spec: gitea-api — Scenario: Obtención de issues de un repositorio → verificado por la nota de spike de gitea-api.
- Spec: gitea-api — Scenario: Registro exitoso de tiempo invertido → verificado por la nota de spike de gitea-api (POST /times).
- Spec: gitea-api — Scenario: Publicación exitosa de comentario en issue → verificado por la nota de spike de gitea-api (POST /comments).
- Spec: gitea-api — Scenario: Error de autenticación mapeado → verificado por la nota de spike de mapeo de errores.
- Spec: gitea-api — Scenario: Error de servidor mapeado → verificado por la nota de spike de reintento para 500.
- Spec: gitea-api — Scenario: Consulta del tiempo acumulado de un issue → verificado por la nota de spike de la unidad timetracking.
- Spec: iced-desktop — Scenario: Apertura de una segunda ventana → verificado por la nota de spike de iced-desktop y el ADR de framework.
- Spec: iced-desktop — Scenario: Cierre independiente de una ventana → verificado por la nota de spike de iced-desktop.
- Spec: iced-desktop — Scenario: Activación del modo siempre arriba → verificado por la nota de spike de iced-desktop.
- Spec: iced-desktop — Scenario: Desactivación del modo siempre arriba → verificado por la nota de spike de iced-desktop.
- Spec: iced-desktop — Scenario: Arrastre de una ventana por el escritorio → verificado por la nota de spike de iced-desktop.
- Spec: iced-desktop — Scenario: Redimensión dentro de límites declarados → verificado por la nota de spike de iced-desktop.
- Spec: iced-desktop — Scenario: Exposición de controles a tecnologías de asistencia → verificado por la nota de spike de accesibilidad.
- Spec: iced-desktop — Scenario: Ejecución de pruebas de interfaz en modo headless → verificado por la nota de spike de iced_test.
- Spec: iced-desktop — Scenario: Generación del paquete instalable → verificado por la nota de spike de empaquetado.
- Spec: adr-registro — Scenario: Creación de una nueva decisión de arquitectura → verificado por el ADR registrado y su identificador único.
- Spec: adr-registro — Scenario: Consulta del historial de decisiones → verificado por el historial de ADR.
- Spec: adr-registro — Scenario: Registro de issue con formato válido → verificado por el ADR de identidad de issue (owner/repo#N).
- Spec: adr-registro — Scenario: Rechazo de identidad de issue inválida → verificado por el ADR de identidad de issue.
- Spec: adr-registro — Scenario: Asociación de tiempo con comentario de issue → verificado por el ADR de mapeo tiempo/comentario.
- Spec: adr-registro — Scenario: Consulta de tiempo vinculado a un comentario → verificado por el ADR de mapeo tiempo/comentario.
- Spec: adr-registro — Scenario: Almacenamiento cifrado de credenciales → verificado por el ADR de almacenamiento de credenciales.
- Spec: adr-registro — Scenario: Recuperación de credenciales por usuario autenticado → verificado por el ADR de almacenamiento de credenciales.
- Spec: matriz-paridad — Scenario: Registro de una capacidad portada → verificado por la matriz de paridad congelada.
- Spec: matriz-paridad — Scenario: Registro de una capacidad adaptada → verificado por la matriz de paridad congelada.
- Spec: matriz-paridad — Scenario: Registro de una capacidad descartada → verificado por la matriz de paridad congelada.
- Spec: matriz-paridad — Scenario: Intento de modificación de alcance congelado sin aprobación → verificado por el procedimiento de congelamiento de alcance.
- Spec: matriz-paridad — Scenario: Cambio de alcance con nueva aprobación documentada → verificado por el procedimiento de aprobación documentada.
- Spec: matriz-paridad — Scenario: Consulta del origen de una clasificación → verificado por la trazabilidad spike/ADR de cada clasificación.

## Success Criteria

- Cada escenario de las cuatro especificaciones tiene un artefacto de descubrimiento correspondiente (nota de spike, ADR o entrada de matriz) que lo cubre.
- Cada ADR contiene título, contexto, decisión, consecuencias, fecha e identificador único, y es consultable en el historial.
- La matriz de paridad clasifica cada capacidad como portada, adaptada o descartada con justificación y trazabilidad al spike o ADR correspondiente.
- El alcance congelado solo cambia con aprobación documentada.
- docs/project-context.md y docs/standards-gaps.md reflejan los hallazgos de los spikes y las decisiones aprobadas.
- No hay código de producto bajo src/ ni dependencias ejecutables pendientes de verificación en este cambio.
