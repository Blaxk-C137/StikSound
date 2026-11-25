# StikSound documentation

Read in this order when starting implementation.

## Plan

| Document | Purpose |
| --- | --- |
| [MASTER-TODO.md](MASTER-TODO.md) | Implementation checklist, phase by phase |
| [roadmap.md](roadmap.md) | Why the work is split this way |
| [risks.md](risks.md) | Risks and the mitigation already chosen |

## Product and requirements

| Document | Purpose |
| --- | --- |
| [product/product-brief.md](product/product-brief.md) | Problem, users, scope, and non-goals |
| [requirements/functional-requirements.md](requirements/functional-requirements.md) | What the product must do |
| [requirements/non-functional-requirements.md](requirements/non-functional-requirements.md) | Quality, privacy, platforms, performance |
| [requirements/acceptance-criteria.md](requirements/acceptance-criteria.md) | When the MVP is done |

## Specifications

| Document | Purpose |
| --- | --- |
| [specifications/platforms.md](specifications/platforms.md) | Windows, macOS, and Chrome boundaries |
| [specifications/user-flows.md](specifications/user-flows.md) | Screens and the primary path |
| [specifications/import.md](specifications/import.md) | Clipboard, files, and drag and drop |
| [specifications/myinstants.md](specifications/myinstants.md) | MyInstants provider contract |
| [specifications/editors.md](specifications/editors.md) | Audio and sticker editing rules |
| [specifications/data-model.md](specifications/data-model.md) | Project, sticker, audio, association |
| [specifications/export.md](specifications/export.md) | WebP, sound bundle, native adapter |
| [specifications/whatsapp.md](specifications/whatsapp.md) | Official sticker limits and sound-sticker facts |
| [specifications/security.md](specifications/security.md) | URL, file, and package validation |
| [specifications/schemas/stiksound-project-1.0.schema.json](specifications/schemas/stiksound-project-1.0.schema.json) | Portable project schema |

## Architecture and decisions

| Document | Purpose |
| --- | --- |
| [architecture/overview.md](architecture/overview.md) | Layers, engines, and shared packages |
| [architecture/repository-layout.md](architecture/repository-layout.md) | Monorepo layout to create in Phase 0 |
| [decisions/001-product-scope.md](decisions/001-product-scope.md) | Name, experience, and MVP width |
| [decisions/002-stack.md](decisions/002-stack.md) | Tauri, Rust, React, Chrome |
| [decisions/003-export-and-whatsapp.md](decisions/003-export-and-whatsapp.md) | Why export is decoupled |
| [decisions/004-extension-boundary.md](decisions/004-extension-boundary.md) | What the extension may and may not do |

## Quality

| Document | Purpose |
| --- | --- |
| [qa/test-plan.md](qa/test-plan.md) | Functional, platform, and security tests |
