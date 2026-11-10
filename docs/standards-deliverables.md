---
description: Standards and conventions for generating .docx deliverables from Markdown sources with the nsoft-specboot deliverable command — pipeline, placeholder tokens, revision rules, diagram assets, and the bilingual en/es contract.
globs:
alwaysApply: true
---
# Deliverable Generation Standards

## Overview

This document defines the normative standard for generating corporate `.docx` deliverables from Markdown source documents via `nsoft-specboot deliverable`. The pipeline converts any Markdown input into a corporate-styled `.docx` with a branded cover, SemVer revision history, embedded diagrams, and a persistent revision manifest. The generation engine and its template twins ship inside the `nsoft-specboot` package and are resolved from the installed package root at run time; no engine file is ever copied into a consumer project.

The directory layout is fixed for every Application Type and is not configurable:
- `deliverables/docs/` — Markdown source documents.
- `deliverables/diagrams/` — rendered diagram assets (PNG).
- `deliverables/out/` — generated `.docx` files and the revision manifest at `deliverables/out/_revisions.yaml`.

Commands run from the git repository root; outside it the command refuses with an English error and writes nothing. Initializing a project never creates a `deliverables/` directory — it is consumer-owned data that appears only on first generation.

## Command Surface

- `nsoft-specboot deliverable generate <input.md> [--bump=patch|minor|major] [--comment=<text>]` renders the Markdown source (e.g. `deliverables/docs/vision.md`) to `deliverables/out/<basename>.docx` and appends the resulting revision to `deliverables/out/_revisions.yaml`. A document with no prior manifest entry is produced at version `0.1.0`.
- `nsoft-specboot deliverable status [<basename>]` reads the manifest and prints the current version and history rows for every recorded document or one named basename. It is strictly read-only: it never generates, writes, or modifies a file, and reports a missing manifest or unknown basename as a readable English notice with exit `0`.
- `--comment=<text>` is recorded verbatim as the revision description (never translated or reformatted); without it the localized default description is used (see the bilingual contract).
- All command output is English regardless of the project language; exit codes are `0` (success) and `1` (failure) only. There is no history-reset flag, subcommand, or configuration key; unknown verbs, flags, or extra positionals are refused with exit `1`.

## Identity Configuration

Generation identity comes only from these keys in `nsoft-project.properties`:

| Key | Required | Meaning |
|---|---|---|
| `deliverable.productName` | yes | Product name shown on the document cover |
| `deliverable.logo` | yes | Repository-relative path to the logo image; no built-in default |
| `deliverable.diagramResolution` | no | Raster size of regenerated diagram assets; default `2048x1320` |
| `language` | no | Existing project language key (`en`/`es`, absent means `en`); selects the template twin |

No default identity is substituted: an unset required key is a generation pre-flight failure. No other configuration key affects the deliverable path layout or identity binding.

## Pre-flight and Failure Semantics

Before generating, the command fails closed on: `pandoc` not resolvable on `PATH`; `deliverable.productName` unset or empty; `deliverable.logo` unset or naming a file that does not exist relative to the repository root. Each refusal prints one English message naming the key or path and the exact remediation, exits `1`, and leaves no output file and no manifest entry. A template placeholder check failure inside the engine is likewise mapped to exit `1` naming the failed placeholder, with no `.docx` written.

A missing diagram asset fails open (see Diagram Assets): it is never a pre-flight failure, and availability of external diagram tooling is never checked by the command.

## Placeholder-Token Map

The template twins carry exactly these seven docxtemplater placeholders, using the non-standard `<` / `>` delimiters so tokens never collide with prose:

| Token | Scope | Resolution |
|---|---|---|
| `<CONTENT>` | Body splice target | The pandoc-converted Markdown body (headings, lists, tables, images) |
| `<DOCUMENT_TITLE>` | Cover | First top-level `#` heading of the source Markdown, else the input basename |
| `<PRODUCT_NAME>` | Cover | `deliverable.productName` |
| `<PRODUCT_LOGO>` | Cover | Image at `deliverable.logo`, embedded via the docx image module |
| `<REVISION>` | Cover + history rows | Current SemVer (the cover pairs it with the twin's baked `Version`/`Versión` label) |
| `<REVISION_DATE>` | History rows | ISO date (`YYYY-MM-DD`) of the revision |
| `<REVISION_COMMENT>` | History rows | Explicit `--comment` text or the localized default description |

The English and Spanish twins expose an identical placeholder inventory; a token present in one twin and absent in the other is a parity violation caught by the shipped-surface tests before release.

The twins also ship two of these tokens in the document metadata: the core property `dc:title` is the literal `<DOCUMENT_TITLE>` in `docProps/core.xml`, and the custom document property `ProductName` is the literal `<PRODUCT_NAME>` in `docProps/custom.xml`. The pipeline substitutes both after rendering — docxtemplater never processes `docProps` — with the resolved document title and `deliverable.productName`, the same values the body tokens receive, so no literal token remains in the produced metadata parts. `dc:creator` is never modified: whatever value the template carries is preserved. The substitution is language-independent (both twins ship the same two tokens) and never alters the document body content or the token map above.

## Heading-Level Body Paragraph Styles

Plain body paragraphs of the generated document carry an explicit, defined paragraph style chosen by the nearest preceding heading level, so a section reads uniformly and matches the project body look rather than pandoc's inconsistent first-paragraph/subsequent-paragraph styling. The mapping is fixed and language-independent:

| Nearest preceding heading | Body paragraph style |
|---|---|
| H1, or before any heading | `Normal` (the default paragraph style) |
| H2 | `Normal 2` |
| H3 | `Normal 3` |
| H4 | `Normal 4` |
| H5 or deeper | `Normal 5` |

The walk is a post-render step, executed after the docxtemplater fill and before the output is written, scoped strictly to the document body. It is never applied to: heading paragraphs themselves; list items (they keep their list/numbering style); table-cell paragraphs; code blocks (`Source Code`, plus inline `Verbatim Char` spans, which are run properties the walk never touches); and figure/caption paragraphs (`Captioned Figure`, `Image Caption`). The level→style mapping is resolved from the twin's `word/styles.xml` by style NAME (`Normal 2`..`Normal 5`), not by the numeric style id, so an editor that renumbers style ids cannot break it; a name that is missing is skipped (those paragraphs keep pandoc's style, reported on stderr) and never crashes generation. The step rewrites only `<w:pStyle>` references in `word/document.xml`; it leaves the placeholder-token map, the headers/footers, the styles part, and the document metadata untouched.

Because the walk reassigns every plain body paragraph to a `Normal`-derived level style, the shipped `Body Text` style is additionally aligned to the `Normal` look — no stale Times New Roman font and no residual left indent — so no body paragraph can render with that stale look even if some other reference to it remains. Both twins ship `Normal 2` alongside the existing `Normal 3`/`Normal 4`/`Normal 5` (each `basedOn Normal`, indents mirroring the same-numbered heading, `Normal 2` with no indent like Heading 2).

## Versioning and the Revision Manifest

- First generation of a document records `0.1.0`; regeneration applies a SemVer bump (`patch` default, or `minor`/`major` via `--bump`) to the recorded current version.
- `deliverables/out/_revisions.yaml` is the single source of truth, keyed by output basename, history ordered oldest-first with the last entry carrying the current version:

```yaml
# Revision manifest — auto-generated by generate.mjs
vision.docx:
  version: "0.1.0"
  history:
    - version: "0.1.0"
      date: "2026-09-23"
      comment: "Initial version"
```

- The manifest is append-only: every successful generation appends exactly one entry and every pre-existing entry remains byte-identical. Nothing may reset, truncate, or rewrite recorded history; never hand-edit the file (a corrupt manifest aborts generation with a clear error).
- The revision history table inside the `.docx` renders one row per history entry, located by the language-aware history-header anchor described below.

## Diagram Assets

Each Mermaid block in the source is keyed by an explicit `<!-- diagram: <key> -->` marker immediately before the fence, or by the nearest preceding heading normalized to kebab-case. Keys resolve against `deliverables/diagrams/` with the configured raster (default `2048x1320`):

1. `<key>.visual-check.<res>.light.png` (preferred), falling back to the `.dark.png` variant of the same raster.
2. If the key carries a version suffix (e.g. `key.v2`), the suffix is stripped and the convention retried.
3. Broad search: any `<key>*visual-check*.png`, preferring the highest `.vN` version, then its light variant, then its dark variant, then any asset of that version.

When no asset resolves, generation does not fail: the document embeds the placeholder text `*[Missing diagram: <key>]*` at that position, the command prints an English warning naming the key, and generation still completes with exit `0`. Resolved diagrams embed at a constant width equal to the page text width (~18.73 cm). Diagram PNGs are versioned project inputs, required for clean-checkout regeneration.

## archify — Optional External Diagram Tool

`archify` (<https://github.com/tt-a1i/archify>) is the external diagram tool the deliverable workflow uses, when present, to regenerate missing or stale diagram assets. It is a documented external prerequisite of the workflow, never a prerequisite of generation: `nsoft-specboot` never vendors, seeds, bundles, or installs it through any command, and generation proceeds without it, leaving the affected diagram as a `*[Missing diagram: <key>]*` placeholder. Detection is agent-side, defined in the `nssx-deliverable` skill: when an asset is missing the skill probes for `archify` and either delegates asset regeneration before generating, or prints this documented install hint and still completes generation.

The documented English install hint is: "archify is not installed — optional diagram regeneration is unavailable; install it from https://github.com/tt-a1i/archify to produce the missing diagram asset (see the package README Prerequisites)."

## Bilingual en/es Contract

- The existing `language` key of `nsoft-project.properties` selects the template twin: `en` selects the English twin, `es` the Spanish twin, absent behaves as `en`. There is no separate deliverable language key and the accepted values are never redefined.
- Strings baked into each twin's `document.xml`: the cover version line reads `Version X.X.X` (en) / `Versión X.X.X` (es), and the revision-history header reads `REVISION | DATE | DESCRIPTION` (en) / `REVISIÓN | FECHA | DESCRIPCIÓN` (es).
- Strings that are engine constants, localized by the received `language`: the default revision descriptions — `Initial version` / `Versión inicial` for a first revision and `Regeneration <bump>` / `Regeneración <bump>` for a bumped one.
- The history-table locator is language-aware and anchors on the header's THIRD-CELL word, prefix-matched against each row's concatenated `<w:t>` text (`DESCRIPTION` for en, the accent-free `DESCRIPCI` for es), never by full-literal equality and never against raw XML: the header is run-split (e.g. `DES`+`CR`+`IP`+`TI`+`ON`), so the contiguous literal never appears in `document.xml`. The third-cell word is the only header text absent from the `<REVISION>` / `<REVISION_DATE>` / `<REVISION_COMMENT>` token row, so the anchor locates the header row and never the token row; a first-cell anchor would be ambiguous because both localized headers now begin with the `REVISI` prefix shared with that token row. The same header-row lookup thresholds the pruning of residual empty history rows.
- Command output and shipped artifacts — the `nssx-deliverable` skill text and this standard — are English-only whatever `language` is; only strings embedded in the generated document are localized.

## Spanish Terminology

Spanish deliverables (`language=es` output) use natural Rioplatense/Latin American Spanish in prose, not English text reproduced word-for-word. Authors and agents writing the Spanish deliverable Markdown correct the following three families:

(A) Anglicisms in prose — English words used inside running Spanish text:
- `el review` → `la revisión`
- `el deadline` → `la fecha límite`
- `el feedback` → `los comentarios` / `la retroalimentación`
- `el deploy` → `el despliegue`
- `setear` / `loggear` → `configurar` / `registrar`

(B) Calqued idioms — English structures copied literally into Spanish:
- `eventualmente` meaning "eventually" → `finalmente` / `con el tiempo`
- `de acuerdo a` → `según` / `de acuerdo con`
- `aplicar para un puesto` → `postularse para un puesto`
- `realizar una actualización` → `actualizar`
- `está en condiciones de operar` → `puede operar`

(C) Peninsular or bureaucratic register — forms out of place in Latin American business prose:
- `vosotros entregáis` → `ustedes entregan`
- `el fichero` → `el archivo`
- `el ordenador` → `la computadora`
- `a nivel de arquitectura, …` (filler) → `en la arquitectura, …`
- `esta decisión impacta en los costos` → `esta decisión afecta los costos`
- `Por medio de la presente se comunica que …` → state the fact directly

Do NOT touch: sanctioned English tokens (`<PRODUCT_NAME>` and friends), identifiers, file names, code, API and protocol names, and product or machine names stay exactly as they are in Spanish deliverables, per the project language rule (localization applies to prose, never to technical artifacts).

## One-Time Orphan-Copy Convergence

Before this capability shipped, a stale machine-global `nssx-deliverable` skill copy carrying hardcoded paths to another project could exist under the global skills mirror. Its retirement requires no user action beyond a regular `init`: the roster seed overwrites the machine-global canonical root always (never merged), and the link machinery replaces pre-existing real-directory mirror conflicts with canonical content, reporting every resolution in the init summary. The first global `init` after the package ships therefore converges or conflict-resolves the orphan, leaving exactly one authoritative roster skill that drives only `nsoft-specboot deliverable`.

## Exclusions

- PDF history files under `deliverables/out/history/*.pdf` are vestigial manual artifacts of the pre-absorption pipeline; the shipped pipeline neither reads nor produces them.
- No history-reset surface exists (see the command surface).
- Consumer template overrides are not offered: the twins are package-internal assets; only the identity keys above are configurable.
- No configuration key relocates `deliverables/{docs,diagrams,out}` or the manifest, and no new configuration key joins the deliverable path beyond the declared `deliverable.*` set and the reused `language` key.
