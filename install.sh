#!/bin/bash
#
# Story Launcher Installer
# Usage: curl -sL https://raw.githubusercontent.com/joyrider00/story-launcher/main/install.sh | bash
#

set -e

REPO="joyrider00/story-launcher"
APP_NAME="Story Launcher"
DMG_PATTERN="Story.Launcher_.*_aarch64.dmg"

echo "Installing $APP_NAME..."

# Get the latest release DMG URL
echo "Finding latest release..."
RELEASE_URL=$(curl -sL "https://api.github.com/repos/$REPO/releases/latest" | \
    grep "browser_download_url.*$DMG_PATTERN" | \
    head -1 | \
    cut -d '"' -f 4)

if [ -z "$RELEASE_URL" ]; then
    echo "Error: Could not find DMG in latest release"
    exit 1
fi

echo "Downloading from: $RELEASE_URL"

# Create temp directory
TEMP_DIR=$(mktemp -d)
DMG_PATH="$TEMP_DIR/StoryLauncher.dmg"

# Download DMG
curl -L -o "$DMG_PATH" "$RELEASE_URL"

# Mount DMG
echo "Mounting DMG..."
MOUNT_POINT=$(hdiutil attach "$DMG_PATH" -nobrowse -quiet | grep "/Volumes" | cut -f 3-)

if [ -z "$MOUNT_POINT" ]; then
    echo "Error: Failed to mount DMG"
    rm -rf "$TEMP_DIR"
    exit 1
fi

# Copy app to Applications
echo "Installing to /Applications..."
if [ -d "/Applications/$APP_NAME.app" ]; then
    echo "Removing existing installation..."
    rm -rf "/Applications/$APP_NAME.app"
fi

cp -R "$MOUNT_POINT/$APP_NAME.app" "/Applications/"

# Remove quarantine flag
echo "Removing quarantine flag..."
xattr -cr "/Applications/$APP_NAME.app"

# Unmount DMG
echo "Cleaning up..."
hdiutil detach "$MOUNT_POINT" -quiet
rm -rf "$TEMP_DIR"

echo ""
echo "✓ $APP_NAME installed successfully!"
echo ""

# Launch the app
echo "Launching $APP_NAME..."
open "/Applications/$APP_NAME.app"
