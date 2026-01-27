# Release Process for Story Launcher

This document explains how to create and publish new releases of Story Launcher.

## Prerequisites

### 1. Generate Tauri Signing Keys

The updater requires a signing key pair to verify updates. Generate one using:

```bash
npm run tauri signer generate -- -w ~/.tauri/story-launcher.key
```

This creates:
- `~/.tauri/story-launcher.key` - Private key (keep secret!)
- `~/.tauri/story-launcher.key.pub` - Public key

### 2. Configure GitHub Secrets

Add these secrets to your GitHub repository (Settings > Secrets and variables > Actions):

#### Required for Updates
- `TAURI_SIGNING_PRIVATE_KEY` - Contents of `~/.tauri/story-launcher.key`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` - Password used when generating the key (if any)

#### Optional for Code Signing (macOS)
For notarized releases that don't show security warnings:

- `APPLE_CERTIFICATE` - Base64-encoded Developer ID certificate (.p12)
- `APPLE_CERTIFICATE_PASSWORD` - Password for the certificate
- `APPLE_SIGNING_IDENTITY` - e.g., "Developer ID Application: Your Name (TEAM_ID)"
- `APPLE_ID` - Your Apple ID email
- `APPLE_PASSWORD` - App-specific password from appleid.apple.com
- `APPLE_TEAM_ID` - Your Apple Developer Team ID
- `KEYCHAIN_PASSWORD` - Any password for the temporary keychain

### 3. Update the Public Key

Replace `UPDATER_PUBKEY_PLACEHOLDER` in `src-tauri/tauri.conf.json` with your public key:

```json
{
  "plugins": {
    "updater": {
      "endpoints": [
        "https://github.com/story-inc/story-launcher/releases/latest/download/latest.json"
      ],
      "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6..."
    }
  }
}
```

## Creating a Release

### Method 1: Using Git Tags (Recommended)

1. **Update the version** in these files:
   - `src-tauri/tauri.conf.json` - `version` field
   - `src/App.tsx` - `APP_VERSION` constant
   - `package.json` - `version` field

2. **Commit the version bump:**
   ```bash
   git add -A
   git commit -m "Bump version to X.Y.Z"
   ```

3. **Create and push a tag:**
   ```bash
   git tag vX.Y.Z
   git push origin main --tags
   ```

4. **Monitor the release:**
   - Go to GitHub Actions to watch the build progress
   - Once complete, the release will be created as a draft
   - Review and publish the release

### Method 2: Manual Workflow Dispatch

1. Go to GitHub Actions > Release workflow
2. Click "Run workflow"
3. Enter the version number (e.g., `0.2.0`)
4. Click "Run workflow"

## What the Release Workflow Does

1. **Builds for macOS:**
   - Apple Silicon (aarch64)
   - Intel (x86_64)

2. **Creates artifacts:**
   - `.dmg` installer files
   - `.app.tar.gz` for updates
   - `.app.tar.gz.sig` signature files

3. **Code signs** (if certificates are configured):
   - Signs the app with your Developer ID
   - Notarizes with Apple for Gatekeeper approval

4. **Generates `latest.json`:**
   - Update manifest for Tauri's updater
   - Contains URLs and signatures for each platform

5. **Creates GitHub Release:**
   - Attaches all build artifacts
   - Includes the update manifest

## Update Flow

When users launch Story Launcher:

1. App checks `latest.json` from GitHub releases
2. Compares version with current version
3. If newer, downloads the update in background
4. Shows a subtle "Update ready" banner
5. User clicks "Restart Now" to apply update
6. App relaunches with new version

## Troubleshooting

### Updates not working

1. Verify `latest.json` is accessible at the endpoint URL
2. Check that the public key in `tauri.conf.json` matches the private key used to sign
3. Ensure signature files (`.sig`) were generated during build

### Code signing issues

1. Verify certificate is valid: `security find-identity -v -p codesigning`
2. Check notarization status: `xcrun stapler validate "Story Launcher.app"`
3. Ensure all Apple credentials are correct in GitHub secrets

### Build failures

1. Check GitHub Actions logs for specific errors
2. Verify Rust targets are installed: `rustup target list --installed`
3. Ensure npm dependencies are up to date: `npm ci`

## Version Numbering

Follow semantic versioning (SemVer):

- **MAJOR** (X.0.0) - Breaking changes
- **MINOR** (0.X.0) - New features, backward compatible
- **PATCH** (0.0.X) - Bug fixes, backward compatible

Example progression: `0.1.0` → `0.1.1` → `0.2.0` → `1.0.0`
