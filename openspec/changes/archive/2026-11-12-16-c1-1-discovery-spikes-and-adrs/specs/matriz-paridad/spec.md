## Purpose
Esta especificación define el comportamiento de la matriz de paridad del puerto, que clasifica cada capacidad del cronómetro de escritorio como portada, adaptada o descartada y documenta el alcance del puerto.

## ADDED Requirements

### Requirement: Clasificación de capacidades en la matriz de paridad
La matriz SHALL clasificar cada capacidad del puerto como portada, adaptada o descartada. La matriz MUST incluir para cada capacidad su nombre, clasificación y justificación de la decisión.

#### Scenario: Registro de una capacidad portada
- **WHEN** se clasifica una capacidad como portada en la matriz de paridad
- **THEN** la matriz registra la capacidad con clasificación "portada" y su justificación

#### Scenario: Registro de una capacidad adaptada
- **WHEN** se clasifica una capacidad como adaptada en la matriz de paridad
- **THEN** la matriz registra la capacidad con clasificación "adaptada", su justificación y las adaptaciones requeridas

#### Scenario: Registro de una capacidad descartada
- **WHEN** se clasifica una capacidad como descartada en la matriz de paridad
- **THEN** la matriz registra la capacidad con clasificación "descartada" y su justificación

### Requirement: Congelamiento del alcance del puerto
La matriz SHALL congelar el alcance del puerto una vez aprobada, de modo que ninguna capacidad cambie de clasificación sin una revisión explícita. La matriz MUST rechazar cambios al alcance congelado sin una nueva aprobación documentada.

#### Scenario: Intento de modificación de alcance congelado sin aprobación
- **WHEN** se intenta cambiar la clasificación de una capacidad después de que el alcance está congelado y no hay nueva aprobación
- **THEN** la matriz rechaza el cambio y conserva la clasificación anterior

#### Scenario: Cambio de alcance con nueva aprobación documentada
- **WHEN** se aprueba documentadamente un cambio de clasificación de una capacidad en alcance congelado
- **THEN** la matriz actualiza la clasificación y registra la nueva aprobación

### Requirement: Trazabilidad de la matriz de paridad
La matriz SHALL mantener trazabilidad entre cada clasificación y el spike o ADR que fundamenta la decisión. La matriz MUST permitir consultar, para cada capacidad, el origen de su clasificación.

#### Scenario: Consulta del origen de una clasificación
- **WHEN** se consulta el origen de la clasificación de una capacidad en la matriz
- **THEN** la matriz devuelve el spike o ADR que fundamenta la decisión
