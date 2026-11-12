# adr-registro Specification

## Purpose
Esta especificación define el comportamiento del registro de decisiones de arquitectura, que documenta la selección del framework, la identidad de los issues como owner/repo#N, el mapeo entre tiempo y comentarios y el almacenamiento de credenciales.

## Requirements

### Requirement: Registro de decisiones de arquitectura (ADR)
El registro SHALL almacenar cada decisión de arquitectura como un documento independiente con título, contexto, decisión, consecuencias y fecha. El registro MUST asignar un identificador único a cada ADR y permitir consultar el historial completo.

#### Scenario: Creación de una nueva decisión de arquitectura
- **WHEN** se registra una nueva ADR con título, contexto, decisión y consecuencias
- **THEN** el registro almacena el documento con un identificador único y lo incluye en el historial

#### Scenario: Consulta del historial de decisiones
- **WHEN** se solicita el historial completo de ADR registradas
- **THEN** el registro devuelve todas las decisiones ordenadas por fecha de creación

### Requirement: Identidad de issue (owner/repo#N)
El registro SHALL identificar cada issue de forma inequívoca usando el formato owner/repo#N. El registro MUST rechazar identificadores de issue que no cumplan este formato.

#### Scenario: Registro de issue con formato válido
- **WHEN** se registra una relación con un issue usando el formato owner/repo#N
- **THEN** el registro acepta la identidad y la asocia a la decisión correspondiente

#### Scenario: Rechazo de identidad de issue inválida
- **WHEN** se intenta registrar un issue con un identificador que no sigue el formato owner/repo#N
- **THEN** el registro rechaza la entrada y reporta que el formato es inválido

### Requirement: Mapeo entre tiempo y comentarios
El registro SHALL asociar cada entrada de tiempo registrada con el comentario de issue correspondiente. El registro MUST permitir consultar todo el tiempo vinculado a un comentario específico.

#### Scenario: Asociación de tiempo con comentario de issue
- **WHEN** se registra una entrada de tiempo vinculada a un comentario de issue identificado como owner/repo#N
- **THEN** el registro asocia el tiempo a ese comentario y lo recupera al consultar el comentario

#### Scenario: Consulta de tiempo vinculado a un comentario
- **WHEN** se consulta el tiempo total vinculado a un comentario de issue específico
- **THEN** el registro devuelve la suma de todas las entradas de tiempo asociadas a ese comentario

### Requirement: Almacenamiento de credenciales
El registro SHALL almacenar las credenciales de acceso a la API de Gitea de forma cifrada en el almacén de credenciales del sistema operativo. El registro MUST recuperar las credenciales solo para el usuario autenticado y nunca exponerlas en texto plano.

#### Scenario: Almacenamiento cifrado de credenciales
- **WHEN** se almacenan las credenciales de acceso a la API de Gitea
- **THEN** el registro las cifra y las guarda en el almacén de credenciales del sistema operativo

#### Scenario: Recuperación de credenciales por usuario autenticado
- **WHEN** el usuario autenticado solicita las credenciales almacenadas
- **THEN** el registro las recupera del almacén y las entrega solo a ese usuario
