# Contributing to Story Launcher

This guide explains how to make updates to Story Launcher using Claude Code (CC).

## Quick Start

1. **Make changes** - Describe what you want in CC
2. **Release** - Tell CC to bump version, commit, tag, and push
3. **Done** - GitHub Actions handles the rest

## Adding or Updating Apps

### Add a Local Tool

```
Add a new local tool called 'Watch Script' that lives at ~/projects/watch-script,
checks for git updates, and runs 'pip install -e .' to install dependencies
```

### Add a Web App

```
Add a new web app called 'Dashboard' that opens https://dashboard.story.inc
with a blue gradient icon
```

### Modify Existing App

```
Update the Resolve Sync Script card to also show the last sync timestamp
```

## Common Updates

| Change | Example CC Prompt |
|--------|-------------------|
| Add local tool | "Add a new local tool called X that lives at ~/projects/X, checks for git updates, and runs Y to install" |
| Add web app | "Add a new web app called X that opens https://x.story.inc" |
| Change UI | "Update the app card design to show X" |
| Fix bug | "Fix the issue where X happens" |
| Add setting | "Add a toggle in Settings for X that does Y" |
| Update tray menu | "Add X to the system tray menu" |

## Releasing a New Version

After making changes, always end your CC session with:

```
Bump the version in package.json and tauri.conf.json to X.X.X,
commit with message "vX.X.X - description of changes",
tag as vX.X.X, and push with tags
```

### Example Release Flow

```
# After adding a new feature:
Bump the version to 0.2.0, commit with message "v0.2.0 - added Watch Script",
tag as v0.2.0, and push with tags
```

## What Happens Automatically

1. **GitHub Action triggers** - Builds for Intel and Apple Silicon Macs
2. **Release created** - DMG installers + update manifest uploaded
3. **Users get updates** - Launcher checks on startup or via "Check for Updates"
4. **One-click install** - User sees prompt → clicks "Restart Now" → done

## Version Numbering

Follow semantic versioning:

- **Patch** (0.1.X) - Bug fixes, small tweaks
- **Minor** (0.X.0) - New features, new apps
- **Major** (X.0.0) - Breaking changes, major redesigns

Examples:
- `0.1.0` → `0.1.1` (bug fix)
- `0.1.1` → `0.2.0` (added new tool)
- `0.2.0` → `1.0.0` (major redesign)

## Project Structure

```
story-launcher/
├── src/                    # React frontend
│   ├── App.tsx            # Main app (UI, state, all components)
│   └── assets/            # Images, logos
├── src-tauri/
│   ├── src/lib.rs         # Rust backend (commands, tray, window)
│   ├── tauri.conf.json    # App config (version, updater, window)
│   ├── capabilities/      # Permissions
│   └── icons/             # App and tray icons
├── .github/workflows/
│   └── release.yml        # Build and release automation
├── RELEASE.md             # Detailed release instructions
└── CONTRIBUTING.md        # This file
```

## Key Files to Know

| File | What it does |
|------|--------------|
| `src/App.tsx` | All UI components, tool cards, settings page |
| `src-tauri/src/lib.rs` | Backend commands (check updates, launch tools), tray menu |
| `src-tauri/tauri.conf.json` | App version, updater config, window settings |
| `package.json` | npm dependencies, version (keep in sync with tauri.conf.json) |

## Tips for CC Prompts

### Be Specific
```
# Good
"Add a local tool called 'Media Encoder' at ~/projects/media-encoder
that shows version from VERSION file and runs './install.sh' to update"

# Vague
"Add a new tool"
```

### Reference Existing Patterns
```
"Add a new web app like Spellbook but called 'Analytics'
that opens https://analytics.story.inc"
```

### Combine Changes
```
"Add a new tool called X, update the sidebar to show tool count,
and add a 'Refresh All' button to the apps page"
```

## Troubleshooting

### Build Failed in GitHub Actions
- Check the Actions tab for error logs
- Ensure `TAURI_SIGNING_PRIVATE_KEY` secret is set
- Verify version numbers match in package.json and tauri.conf.json

### Updates Not Working for Users
- Verify latest.json exists in the GitHub release
- Check that the public key in tauri.conf.json matches the signing key
- Ensure the updater endpoint URL is correct

### Local Development
```bash
npm run tauri dev    # Run in development mode
npm run tauri build  # Build release version
```

## Need Help?

- Check `RELEASE.md` for detailed release process
- Review existing code in `src/App.tsx` for UI patterns
- Look at `src-tauri/src/lib.rs` for backend command examples
