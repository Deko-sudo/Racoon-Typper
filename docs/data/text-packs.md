# Text Packs — Versioned Practice Content Interchange

**Status:** Export, preview, and import of text packs are exposed through the
Texts view. A pack bundles named custom texts under one language so practice
material can be shared between users and installations.

## Schema v1 (`racoon-typper-text-pack`)

```json
{
  "format": "racoon-typper-text-pack",
  "schema_version": 1,
  "application_version": "1.2.0",
  "exported_at": "2026-08-25T00:00:00+00:00",
  "language": "en",
  "texts": [ { "name": "Home row drill", "text": "asdf jkl;" } ]
}
```

Hard bounds (enforced before any write): input ≤ 4 MiB, ≤ 500 entries,
entry names ≤ 200 chars, entry texts ≤ 10 000 chars — the same limits the
editor enforces, so an imported pack never contains what the UI could not
author. Names are matched case-insensitively when merging.

## Import sources and mapping rules

| Format | Selected by | Mapping |
|---|---|---|
| JSON | `.json`, or auto-detected `{` prefix | Must match the schema above exactly; foreign formats and future versions are rejected. |
| Text blocks | `.txt`/`.md`, or auto fallback | Blank-line separated blocks. Multi-line block → first line is the name, the rest is the text. Single-line block → deterministic name `Text N`. |
| TSV | `.tsv`, or auto-detected tabs | One entry per line; column 1 = name, remaining columns joined with spaces = text (Anki tab-separated export). Single-column lines get `Text N` names. |
| CSV | `.csv` only — never auto-guessed | RFC4180 rows (quoted fields, `""` escapes) mapped exactly like TSV. |

Plain-text sources carry no language metadata and default to `en`; JSON packs
must state their language.

## Policies

- **Merge** skips entries whose normalized `(language, name)` already exists;
  duplicates inside one pack collapse to the first occurrence.
- **Replace** deletes every custom text of the pack language, then inserts the
  pack. It is destructive but strictly scoped to that language and requires an
  explicit acknowledgement in the UI.

Both preview and apply run the same parser and validation; apply executes in a
single transaction, so a failure rolls back every write. Import commands are
gated on startup recovery being ready.
