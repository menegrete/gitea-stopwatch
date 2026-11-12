## 1. Setup y rama de características

- [x] 1.1 Crear y cambiar a la rama feature/16-c1-1-discovery-spikes-and-adrs (ya completada durante el bootstrap de nssx-start)

## 2. Revisión y actualización de pruebas unitarias existentes (MANDATORY — NO APLICABLE)

- [x] 2.1 Confirmar que no existe código de producto bajo src/ ni pruebas unitarias que actualizar (test-plan: fuera de scope, no hay módulos ejecutables) — NO APLICABLE: este cambio no entrega código de producto bajo src/ ni dependencias ejecutables (discovery/spikes/ADR/documentación)

## 3. Ejecución de pruebas unitarias y verificaciones de cobertura (MANDATORY — NO APLICABLE)

- [x] 3.1 Confirmar no aplicable: no hay módulos con lógica ejecutable bajo src/ susceptibles de cobertura unitaria (test-plan) — NO APLICABLE: este cambio no entrega código de producto bajo src/ ni módulos con lógica ejecutable susceptibles de cobertura unitaria (discovery/spikes/ADR/documentación, fuera de scope del test-plan)

## 4. Pruebas end-to-end de la aplicación de escritorio enviada (MANDATORY — NO APLICABLE / AGENTE DEBE VERIFICAR NO APLICABILIDAD)

- [ ] 4.1 Verificar no aplicable: no existe binario ni aplicación empaquetada que ejecutar en este cambio (test-plan)

## 5. Ejecución del spike de la API de Gitea (gitea-api)

- [ ] 5.1 Ejecutar autenticación con PAT, GET /user y obtención de issues; registrar notas incl. límites de API (espec gitea-api)
- [ ] 5.2 Ejecutar POST /times, POST /comments, mapeo de errores y unidad timetracking; registrar notas (espec gitea-api)

## 6. Ejecución del spike de escritorio Iced (iced-desktop)

- [ ] 6.1 Ejecutar ventanas múltiples, siempre arriba, arrastre y redimensionado; registrar brechas del framework (espec iced-desktop)
- [ ] 6.2 Ejecutar accesibilidad, iced_test headless y empaquetado; registrar brechas del framework (espec iced-desktop)

## 7. Redacción y aprobación de ADR (adr-registro)

- [ ] 7.1 Redactar ADR de selección de framework (crate HTTP, ventana múltiple) (D1, diseño)
- [ ] 7.2 Redactar ADR de identidad de issue como owner/repo#N (D3, diseño)
- [ ] 7.3 Redactar ADR de mapeo tiempo/comentario (diseño)
- [ ] 7.4 Redactar ADR de almacenamiento de credenciales cifrado (D4, diseño)
- [ ] 7.5 Aprobar los ADR registrados (espec adr-registro)

## 8. Congelamiento de la matriz de paridad (matriz-paridad)

- [ ] 8.1 Clasificar capacidades como portada, adaptada o descartada con justificación y trazabilidad al spike o ADR (D6, diseño)
- [ ] 8.2 Congelar el alcance del puerto y documentar el procedimiento de cambio con aprobación documentada (espec matriz-paridad)

## 9. Actualización de documentación técnica (MANDATORY)

- [ ] 9.1 Actualizar docs/project-context.md con hallazgos de los spikes y ADR aprobados (proposal, diseño)
- [ ] 9.2 Actualizar docs/standards-gaps.md con desviaciones y gaps observados frente a los estándares (diseño)

## 10. Verificación documental de cobertura de objetivos (test-plan)

- [ ] 10.1 Verificar que cada escenario de las cuatro especificaciones tiene un artefacto de descubrimiento correspondiente (test-plan Cobertura)
- [ ] 10.2 Verificar que cada ADR contiene título, contexto, decisión, consecuencias, fecha e identificador único (espec adr-registro)
- [ ] 10.3 Verificar trazabilidad de cada clasificación de la matriz de paridad al spike o ADR (espec matriz-paridad)
