# gitea-api Specification

## Purpose
Este especificación define el comportamiento del cliente de la API de Gitea, que autentica con token de acceso personal, consulta al usuario autenticado, obtiene issues, registra tiempo y comentarios, mapea errores de respuesta y expone una unidad de control de tiempo.

## Requirements

### Requirement: Autenticación con token de acceso personal
El cliente SHALL autenticar cada solicitud a la API de Gitea usando un token de acceso personal (PAT) enviado en el encabezado de autorización. El cliente MUST rechazar cualquier solicitud cuando no se haya configurado un PAT válido.

#### Scenario: Solicitud autenticada con PAT válido
- **WHEN** se configura un PAT válido y se ejecuta una solicitud a la API
- **THEN** la solicitud se envía con el encabezado Authorization: token <PAT> y el servidor responde con éxito

#### Scenario: Solicitud sin PAT configurado
- **WHEN** no se ha configurado ningún PAT y se intenta ejecutar una solicitud
- **THEN** el cliente falla antes de enviar la solicitud y reporta que falta la credencial de autenticación

### Requirement: Consulta del usuario autenticado (GET /user)
El cliente SHALL consultar el endpoint GET /user de la API de Gitea para obtener el perfil del usuario autenticado. El cliente MUST devolver los datos del usuario cuando la respuesta tenga código de éxito.

#### Scenario: Consulta exitosa del usuario autenticado
- **WHEN** se invoca la consulta de usuario autenticado y la API responde con código 200
- **THEN** el cliente devuelve el perfil del usuario con identificador, nombre de usuario y correo electrónico

### Requirement: Obtención de issues
El cliente SHALL obtener la lista de issues de un repositorio mediante el endpoint correspondiente de la API de Gitea. El cliente MUST filtrar los issues por repositorio y devolver al menos el identificador, título y estado de cada issue.

#### Scenario: Obtención de issues de un repositorio
- **WHEN** se solicitan los issues de un repositorio existente y la API responde con lista de issues
- **THEN** el cliente devuelve cada issue con su identificador, título y estado

### Requirement: Registro de tiempo invertido (POST /times)
El cliente SHALL registrar tiempo invertido en un issue mediante el endpoint POST /times de la API de Gitea. El cliente MUST enviar el identificador del issue y la cantidad de tiempo registrada.

#### Scenario: Registro exitoso de tiempo invertido
- **WHEN** se envía una solicitud POST /times con identificador de issue y tiempo válido
- **THEN** la API acepta el registro y el cliente confirma que el tiempo fue contabilizado

### Requirement: Registro de comentarios en issues (POST /comments)
El cliente SHALL publicar comentarios en un issue mediante el endpoint POST /comments de la API de Gitea. El cliente MUST incluir el identificador del issue y el cuerpo del comentario en la solicitud.

#### Scenario: Publicación exitosa de comentario en issue
- **WHEN** se envía una solicitud POST /comments con identificador de issue y cuerpo de comentario válido
- **THEN** la API crea el comentario y el cliente devuelve el comentario creado con su identificador

### Requirement: Mapeo de errores de la API
El cliente SHALL traducir los códigos de respuesta HTTP de la API de Gitea en errores de dominio del cliente. El cliente MUST distinguir entre errores de autenticación, errores de cliente y errores de servidor.

#### Scenario: Error de autenticación mapeado
- **WHEN** la API responde con código 401 a una solicitud autenticada
- **THEN** el cliente reporta un error de credenciales inválidas o expiradas

#### Scenario: Error de servidor mapeado
- **WHEN** la API responde con código 500 a una solicitud válida
- **THEN** el cliente reporta un error del servidor y permite reintento

### Requirement: Unidad de control de tiempo (timetracking)
El cliente SHALL exponer una unidad de control de tiempo que registre el tiempo asociado a cada issue. El cliente MUST mantener el tiempo acumulado por issue y permitir consultar el total registrado.

#### Scenario: Consulta del tiempo acumulado de un issue
- **WHEN** se han registrado varias entradas de tiempo para un issue y se consulta el total
- **THEN** el cliente devuelve la suma de todo el tiempo registrado para ese issue
