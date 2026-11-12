# ADR-02: Identidad de issue como owner/repo#N

**Estado**: Aceptada  
**Fecha**: 2026-11-12  
**Identificador**: ADR-02

## Contexto

Cada issue debe identificarse de forma inequívaca a través de repositorios, siguiendo el modelo del forge donde un issue pertenece a exactamente un repositorio. Un número de issue simple no es suficiente para dar identidad inequívoca a través de repositorios (D3, diseño).

## Decisión

Identificar cada issue con el formato owner/repo#N. Rechazar identificadores de issue que no cumplan este formato.

## Consecuencias

Positivas: identidad inequívoca a través de repositorios; cumple con el modelo uno issue por repositorio del forge.

Negativas: los identificadores simples de issue (número solo) no son válidos y se rechazan.
