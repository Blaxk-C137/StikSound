# Deploy blaxkstik on Render

blaxkstik runs in the browser. Render hosts the compiled HTML, JavaScript, CSS, icons, and service worker as a Static Site. Media editing stays on the user's device.

## Deploy with the Blueprint

1. Create or use a GitHub/GitLab repository and push the complete project to it. Include the root `package.json`, `package-lock.json`, `render.yaml`, `apps`, and `packages` directories. Build output and `node_modules` do not need to be committed.
2. Open the [Render dashboard](https://dashboard.render.com/) and select **New → Blueprint**.
3. Connect the repository and select the branch containing `render.yaml`.
4. Render reads the Blueprint and lists the **blaxkstik** static site. Click **Deploy Blueprint**.
5. Wait for the build to finish and open the HTTPS URL shown in the site's dashboard. The exact hostname depends on availability; use the URL Render assigns.

The Blueprint installs dependencies, checks TypeScript, builds the browser app, configures the rewrite, and sets the cache headers below. Render supplies HTTPS, which is needed for installation, the service worker, and browser sharing features.

## Create the Static Site manually

Choose **New → Static Site**, connect the repository, and use:

| Setting | Value |
| --- | --- |
| Name | `blaxkstik` |
| Branch | Your deployment branch |
| Root directory | Leave blank |
| Build command | `npm ci --include=dev && npm run build` |
| Publish directory | `apps/web/dist` |

Under **Environment**, add `NODE_VERSION` with value `22`.

Under **Redirects/Rewrites**, add:

| Source | Destination | Action |
| --- | --- | --- |
| `/*` | `/index.html` | Rewrite |

Under **Headers**, add:

| Path | Header | Value |
| --- | --- | --- |
| `/*` | `X-Content-Type-Options` | `nosniff` |
| `/` | `Cache-Control` | `no-cache` |
| `/index.html` | `Cache-Control` | `no-cache` |
| `/sw.js` | `Cache-Control` | `no-cache` |
| `/manifest.webmanifest` | `Cache-Control` | `no-cache` |
| `/assets/*` | `Cache-Control` | `public, max-age=31536000, immutable` |

Click **Create Static Site**. Subsequent deployments use the same build and publish settings.

## Deploy as a Web Service

A **Web Service** runs the included Node server. Use these settings for that service type:

| Setting | Value |
| --- | --- |
| Runtime | Node |
| Root directory | Leave blank |
| Build command | `npm ci --include=dev && npm run build` |
| Start command | `npm start` |
| Environment variable | `NODE_VERSION=22` |

The server serves `apps/web/dist`, supports app URL reloads, and listens on `0.0.0.0` with Render's automatic `PORT`. Media editing still runs on the user's device. Upload or push the updated root `package.json` and `tools` directory before redeploying.

The Static Site Blueprint above uses the build command and publish directory; it does not need a start command.

## Verify the deployment

1. Open the deployed URL and check that the page title and editor header say **blaxkstik**.
2. Choose an image and a local audio file, select 5 or 10 seconds, and preview or save the result.
3. Use the menu to save a `.blaxkstik.zip` project, reset the editor, and reopen the archive.
4. Reload the page and check that the editor loads. The rewrite also lets an app URL load after a refresh.
5. On mobile, use **Install app** or Safari's **Add to Home Screen** to install blaxkstik.

For a custom domain, add it in the site's **Settings → Custom Domains** and follow the DNS records Render provides.

## Troubleshooting

- **Build cannot find `package.json`:** keep the Root Directory field blank. The npm workspace must be installed from the repository root.
- **TypeScript or Vite is missing:** use the full build command above. `--include=dev` installs the build tools even if the environment is configured for production.
- **Page is blank or assets return 404:** publish `apps/web/dist` and use the normal root-based build. A GitHub Pages build uses a different base path and should not be uploaded to Render.
- **An older app name still appears after an update:** close the installed app, reopen the site online, and reload. If needed, remove the existing home-screen shortcut and install it again.
- **MyInstants links do not work:** download the sound from MyInstants first, then choose the downloaded audio file in blaxkstik.
- **Video sharing is unavailable:** download the file and share it from your device. Browser sharing and codecs vary by device.
