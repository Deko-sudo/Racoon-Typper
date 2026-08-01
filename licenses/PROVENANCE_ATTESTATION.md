# Project Content Provenance Record

<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- Copyright 2026 Racoon Typper Contributors -->

This record documents the provenance basis for project-owned content shipped by Racoon Typper. It is an engineering record, not a substitute for maintainer or legal sign-off.

## Project-owned content

The following content is marked Apache-2.0 in the asset inventory:

- theme CSS and metadata under `resources/themes/racoon_*`;
- the source SVG and generated PNG icon sizes under `crates/app/icons/`;
- quote packs under `resources/quotes/`;
- course packs under `resources/courses/`.

The word packs, course packs, and quote packs were generated specifically for Racoon Typper using GLM-5.2 and GPT-5.6 (Codex) during project development. They were not imported from, copied from, or derived from an external repository, website, dataset, book, or application. The themes and icon source were created in this repository; PNGs are mechanical outputs of the SVG source.

## Repository-authored attestation

Word packs under `resources/words/`, course packs under `resources/courses/`, and quote packs under `resources/quotes/` are attributed in the machine-readable inventory to Racoon Typper Contributors. The project generation record identifies GLM-5.2 and GPT-5.6 (Codex) as the generation tools, and the repository contains no external source or attribution. The relevant source commits are:

- `7b32a56` — initial word packs, quote packs, and the first course resources;
- `e533fa3` — course foundation and course resources;
- `b828e05` — additional language resources during the project license migration.

No external database, upstream theme catalog, copied quotation collection, font, or screenshot is included in the Phase 1 resource set. The former imported theme catalog, its GPL notice/license files, and its importer have been removed. A repository-wide review found no contradictory source reference or imported-content marker.

## Review status

The Phase 1 final review records these packs as original project-owned content and permits them to remain under Apache-2.0. This record is an engineering provenance statement, not independent legal advice; any future imported or externally derived content must be added to the inventory with its exact source and license or removed.
