<div align="center">

<img src="docs/assets/app-icon.svg" alt="blaxkstik app icon" width="100" height="100" />

# blaxkstik

Create short sticker videos with images, GIFs, video clips, and sound — directly in your browser.

[Deploy on Render](docs/deploy-render.md) · [Run locally](#run-locally) · [Technical documentation](docs/README.md)

</div>

## What you can do

- Upload, drop, or paste a PNG, JPG, WebP, GIF, or short video.
- Crop the image to a square and choose a background.
- Add a local MP3, WAV, OGG, M4A, or AAC file, or use a video's audio.
- Select up to **5 or 10 seconds** of sound and preview the result.
- Download or share a video, export a static WebP, or save a ZIP bundle.
- Save and reopen `.blaxkstik.zip` projects. Existing `.stiksound.zip` projects also open.
- Use the app on desktop or mobile, or add it to your home screen as a PWA.

Images and audio are processed on your device. Hosting serves the app files; your media is not uploaded to Render.

Audio from MyInstants must be downloaded first and selected as a local file. MyInstants does not allow direct downloads from other websites through the browser.

Video format support depends on the browser. Current Chrome, Edge, and Safari are recommended. A video shared to WhatsApp is sent as a video; WhatsApp decides whether it can be converted to a sticker.

## Run locally

Install **Node.js 22 or newer** and **npm 10 or newer**, then run these commands from the repository root:

```bash
npm ci
npm run dev
```

Open **http://localhost:5180**. The web app does not require Rust, Tauri, a database, or API keys.

To check and preview the production build:

```bash
npm test
npm run build
npm run preview
```

`npm run build` checks the web app's TypeScript and creates **`apps/web/dist`**. The preview server normally runs at **http://localhost:4173**.

## Deploy on Render

The repository includes a [render.yaml](render.yaml) Blueprint for a **Render Static Site** named **blaxkstik**.

1. Push this project, including `render.yaml` and `package-lock.json`, to your GitHub or GitLab repository.
2. In the [Render dashboard](https://dashboard.render.com/), choose **New → Blueprint** and connect that repository.
3. Render detects `render.yaml`. Review the **blaxkstik** static site and click **Deploy Blueprint**.
4. Open the HTTPS URL Render provides when deployment finishes.

You can also create **New → Static Site** manually with these settings:

| Setting | Value |
| --- | --- |
| Name | `blaxkstik` |
| Branch | The branch containing these changes |
| Root directory | Leave blank (repository root) |
| Build command | `npm ci --include=dev && npm run build` |
| Publish directory | `apps/web/dist` |
| Environment variable | `NODE_VERSION=22` |
| Rewrite | `/*` → `/index.html` |

The Blueprint also sets cache headers for app updates and compiled assets. Full instructions, including the matching manual headers, are in [docs/deploy-render.md](docs/deploy-render.md).

### Render Web Service

For a Render **Web Service**, choose the **Node** runtime and use these settings:

| Setting | Value |
| --- | --- |
| Root directory | Leave blank |
| Build command | `npm ci --include=dev && npm run build` |
| Start command | `npm start` |

`npm start` serves `apps/web/dist` using Render's `PORT` on `0.0.0.0`. Render supplies the port automatically. To try the production server locally, run `npm run build` followed by `npm start`, then open **http://localhost:3000**.

## Install on a phone

Open the deployed HTTPS site:

- **iPhone/iPad:** Safari → Share → **Add to Home Screen**.
- **Android:** Chrome → menu → **Install app** or **Add to home screen**.

The installed app is named **blaxkstik**. The browser handles installation and permissions for sharing files.

## Project layout

```text
apps/web/           React/Vite web app and PWA assets
packages/core/      Project model, validation, and editing use cases
packages/ui/        Shared editor, timeline, and translations
packages/media-web/ Browser media decoding, encoding, and ZIP handling
packages/providers/ MyInstants URL policy and extraction
render.yaml         Render Static Site deployment
tools/server.mjs    Production server for Render Web Services
```

The original desktop/extension sources and Rust crates remain available in `apps/desktop`, `apps/extension`, and `crates`. Normal development and deployment commands target the web app. `npm run build:all` and `npm run typecheck:all` cover all original workspaces.

## Credits and license

Based on [StikSound by SilverFeit](https://github.com/SilverFeit/StikSound). The original technical documentation retains upstream names for reference.

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
