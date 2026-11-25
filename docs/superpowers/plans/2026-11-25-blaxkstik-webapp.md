# blaxkstik Web App Implementation Plan

> **For agentic workers:** Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox syntax for tracking.

**Goal:** Make this project a browser-first app named blaxkstik that the user can deploy on Render.

**Architecture:** Reuse the existing React/Vite browser shell and local media engine. Publish the built files as a Render Static Site; no media server or database is needed.

**Tech Stack:** React 18, TypeScript, Vite 6, npm workspaces, Node.js 22+, Render Static Sites.

**Spec:** The user requested: “change this to be a webapp”, “i want it to be blaxkstik”, and “host it on render add a render stuff so i can use it”.

## Global Constraints

- Use the exact lowercase product name `blaxkstik` in browser UI, metadata, installed app identity, and new project packages.
- Continue to process media locally and use local audio files in the web app.
- Keep the existing project manifest schema readable, including legacy ZIP projects.
- Make `apps/web/dist` the production build output and Render publish directory.
- Use the existing library package names internally; changing these is unnecessary for the product name.

## Review Focus

- Clean npm installs must work from the committed lockfile.
- Deployment must build only the browser app without Rust or Tauri build tools.
- Reloading a URL must load the correct built assets and manifest.
- Saving and reopening a project must retain its media and trim settings.
- The brand and editor controls must fit on a small phone screen.

### Task 1: Make the browser app the default

**Files:** `package.json`, `package-lock.json`, `apps/web/package.json`, `apps/web/vite.config.ts`, `apps/web/index.html`, `.node-version`, `.github/workflows/pages.yml`.

**Interfaces:** Root npm scripts build the `@blaxkstik/web` workspace and publish `apps/web/dist`; existing library workspaces keep their import names.

- [x] Add root dev/build/preview commands and retain optional build:all/typecheck:all commands.
- [x] Use npm and Node.js 22+ consistently; refresh the lockfile.
- [x] Set the production base URL to `/` and make public asset links and worker registration respect the Vite base URL.
- [ ] Verify TypeScript checking and production compilation.

### Task 2: Apply product branding

**Files:** `packages/ui/src/locales/en.json`, `packages/ui/src/locales/es.json`, `packages/ui/src/components/StikSoundMinimalApp.tsx`, `packages/ui/src/styles.css`, `packages/core/src/domain/errors.ts`, `packages/core/src/domain/schema-validator.ts`, `packages/core/src/use-cases/project-controller.ts`, `packages/core/test/core.test.ts`, `packages/media-web/src/web-media-engine.ts`, `apps/web/src/main.tsx`, `apps/web/public/manifest.webmanifest`, `apps/web/public/sw.js`, `apps/web/public/icons/icon-192.png`.

**Interfaces:** New project downloads use `.blaxkstik.zip`; project serialization remains compatible with existing ZIP archives.

- [x] Update visible titles, about text, error guidance, and export names; add a compact header wordmark.
- [x] Update PWA metadata/cache identity and provide a 192px install icon from the existing artwork.
- [x] Update the existing export-name expectations and run the full unit test suite.

### Task 3: Configure and verify Render hosting

**Files:** `render.yaml`, `README.md`, `docs/deploy-render.md`.

**Interfaces:** Render executes `npm ci --include=dev && npm run build` and serves `./apps/web/dist` over HTTPS.

- [x] Add the static-site blueprint, SPA rewrite, Node version, and appropriate cache headers.
- [x] Document Blueprint deployment and matching manual Static Site settings.
- [ ] Verify a clean install, all tests, type checking, and production build.
- [ ] Use available Python Playwright/Chrome to check production page identity, console, image/audio import, download/project reopen, and desktop/phone layouts. Browser plugin is not available in this session.

## Execution notes

- This directory is a downloaded source tree without valid Git metadata. Work in place and keep a baseline source archive under `/tmp`; Git worktree and commit steps do not apply.
- Branding and deployment configuration are reversible changes; validate them with existing tests and production browser checks rather than adding tests that only duplicate configuration text.

## Final verification record

- Render YAML, app/PWA branding, all icon sizes, workspace lock metadata, and original dependency version/integrity preservation checks passed.
- npm's CI lockfile validator passed for 318 entries using verified cached public package metadata. The lockfile was refreshed with npm Arborist without downloading or installing packages.
- Root dev/preview commands correctly forward arguments to the web workspace.
- The independent source review found one project-reopen bug. Both shell and browser file-input openers now share duration-cap restoration.
- Regression tests for real saved ZIP files with both `.blaxkstik.zip` and `.stiksound.zip` filenames failed with `10.0s / 5.0s` before the fix and passed with `10.0s / 10.0s` after it.
- The final full test command ran: 24 tests passed; `packages/media-web/test/media-web.test.ts` and `packages/media-web/test/video-sniff.test.ts` could not load unavailable `mp4-muxer`.
- The production build's TypeScript stage reported only missing `mp4-muxer` and `webm-muxer`. Restricted npm network access prevented installing these packages, so full production compilation remains unverified here.
- Browser/phone QA remains unverified. The reviewer's Chrome launch was denied by the sandbox (`setsockopt`, then SIGTRAP).
- Temporary links to compatible local test tools were used only for validation and removed afterward. No dependency versions were changed to fit the local cache.
- Review scope decisions: original desktop/extension release metadata, sound-bundle imports, and native WhatsApp sticker support remain outside this browser conversion. Their existing behavior is documented rather than changed.
