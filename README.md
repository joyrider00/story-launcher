# Story Launcher

A native macOS app for managing Story tools and quick access to Story web apps.

![Story Launcher Screenshot](docs/screenshot.png)

## Install

```bash
curl -sL https://raw.githubusercontent.com/joyrider00/story-launcher/main/install.sh | bash
```

This downloads the latest release, installs it to `/Applications`, and launches the app.

## Features

- **Local Tools** - Manage and auto-update local development tools like Resolve Sync Script
- **Web Apps** - Quick launch buttons for Spellbook, Story Portal, and other web apps
- **System Tray** - Lives in your menubar for quick access
- **Auto-Updates** - The launcher updates itself automatically when new versions are released
- **Launch at Login** - Optional setting to start with your Mac

## Apps Included

| App | Type | Description |
|-----|------|-------------|
| Resolve Sync Script | Local | Auto-imports files to DaVinci Resolve |
| Spellbook | Web | Story production management platform |
| Story Portal | Web | Team collaboration and resources hub |

## Development

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for instructions on how to add new tools and release updates.

## Release Process

See [RELEASE.md](RELEASE.md) for detailed release instructions.
