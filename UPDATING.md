# Auto-updates (via GitHub Releases)

Mimic checks for updates on launch via the Tauri updater plugin. If a newer
signed release exists, the app shows a banner: **"Update available
v1.0.0 → v1.1.0 · Download and Install"**. Clicking it downloads, installs and
relaunches.

## How it works

- Each release is **cryptographically signed** with `src-tauri/mimic_updater.key`
  (private — never commit it; back it up safely. Lose it and you can never sign
  updates again). The matching public key is baked into `tauri.conf.json`.
- The app reads a `latest.json` manifest from your repo's **latest GitHub
  release**, compares its `version` to the installed one, and offers the update.
- A GitHub Action (`.github/workflows/release.yml`) builds, signs, creates the
  release, uploads the installer, and generates `latest.json` — all from a tag.

## One-time setup

1. **Create the GitHub repo and push** (this project isn't a git repo yet):
   ```bash
   git init && git add . && git commit -m "Mimic"
   git branch -M main
   git remote add origin https://github.com/Yivani/Mimic.git
   git push -u origin main
   ```

2. **Endpoint is already set** in `src-tauri/tauri.conf.json`:
   ```json
   "endpoints": ["https://github.com/Yivani/Mimic/releases/latest/download/latest.json"]
   ```
   This stable URL always resolves to the newest release's `latest.json`.

3. **Add the signing key as repo secrets** (Settings → Secrets and variables →
   Actions → New repository secret):
   - `TAURI_SIGNING_PRIVATE_KEY` → the full contents of
     `src-tauri/mimic_updater.key`
   - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` → empty (the key has no password)

## Publishing a new version

1. Bump the version in **both** `src-tauri/tauri.conf.json` and `package.json`
   (e.g. `1.0.0` → `1.1.0`).
2. Commit, then tag and push:
   ```bash
   git commit -am "v1.1.0"
   git tag v1.1.0
   git push origin main v1.1.0
   ```
3. The Action builds + signs + publishes the release with the installer and
   `latest.json`. Installed apps show the update banner on their next launch.

> The tag (`v1.1.0`) must match the version in the config files.

## Local manual fallback

If you ever want to publish without the Action, build signed locally and
assemble the manifest yourself:
```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content src-tauri\mimic_updater.key -Raw
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""
npm run tauri:build
.\scripts\make-latest-json.ps1 -HostUrl "https://github.com/Yivani/Mimic/releases/download/v1.1.0"
```
Then upload `release\latest.json` and the installer to a GitHub release manually.

## Notes

- Until the endpoint points at a real repo whose latest release has a version
  **higher** than installed, the check fails silently (no banner) — by design.
- macOS/Linux: add those platforms' signed artifacts to extend `latest.json`
  (and add their runners to the workflow).
