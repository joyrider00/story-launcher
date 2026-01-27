import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { relaunch } from "@tauri-apps/plugin-process";
import { openUrl } from "@tauri-apps/plugin-opener";
import { load } from "@tauri-apps/plugin-store";
import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";
import { check } from "@tauri-apps/plugin-updater";
import StoryLogo from "./assets/StoryLogo.svg";

const APP_VERSION = "0.1.0";
const STORE_NAME = "settings.json";

interface ToolStatus {
  installed: boolean;
  local_version: string | null;
  local_commit: string | null;
  remote_commit: string | null;
  has_update: boolean;
  error: string | null;
}

interface UpdateResult {
  success: boolean;
  message: string;
}

interface WebApp {
  name: string;
  description: string;
  url: string;
  gradient: string;
  icon: React.ReactNode;
}

interface Settings {
  autoUpdateOnLaunch: boolean;
  launchAtLogin: boolean;
}

const defaultSettings: Settings = {
  autoUpdateOnLaunch: true,
  launchAtLogin: false,
};

interface AppUpdate {
  version: string;
  downloadedAndReady: boolean;
  downloading: boolean;
  progress: number;
}

type Page = "apps" | "settings";

function Sidebar({
  currentPage,
  onPageChange,
  updateCount
}: {
  currentPage: Page;
  onPageChange: (page: Page) => void;
  updateCount: number;
}) {
  return (
    <div className="w-56 bg-zinc-900 flex flex-col border-r border-zinc-800">
      <div className="p-5 border-b border-zinc-800">
        <img src={StoryLogo} alt="Story Co" className="h-6" />
      </div>

      <nav className="flex-1 p-3">
        <button
          onClick={() => onPageChange("apps")}
          className={`w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-left transition-colors ${
            currentPage === "apps"
              ? "bg-zinc-800 text-white"
              : "text-zinc-400 hover:text-white hover:bg-zinc-800/50"
          }`}
        >
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />
          </svg>
          <span className="flex-1">Apps</span>
          {updateCount > 0 && (
            <span className="min-w-5 h-5 px-1.5 bg-red-500 text-white text-xs font-bold rounded-full flex items-center justify-center">
              {updateCount}
            </span>
          )}
        </button>

        <button
          onClick={() => onPageChange("settings")}
          className={`w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-left transition-colors mt-1 ${
            currentPage === "settings"
              ? "bg-zinc-800 text-white"
              : "text-zinc-400 hover:text-white hover:bg-zinc-800/50"
          }`}
        >
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
          Settings
        </button>
      </nav>

      <div className="p-4 border-t border-zinc-800">
        <div className="text-xs text-zinc-500">Story Launcher v{APP_VERSION}</div>
      </div>
    </div>
  );
}

function Toggle({
  enabled,
  onChange,
  disabled = false
}: {
  enabled: boolean;
  onChange: (value: boolean) => void;
  disabled?: boolean;
}) {
  return (
    <button
      onClick={() => !disabled && onChange(!enabled)}
      disabled={disabled}
      className={`relative w-11 h-6 rounded-full transition-colors ${
        disabled ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer'
      } ${enabled ? 'bg-blue-600' : 'bg-zinc-700'}`}
    >
      <span
        className={`absolute top-1 w-4 h-4 rounded-full transition-all ${
          enabled ? 'right-1 bg-white' : 'left-1 bg-zinc-400'
        }`}
      />
    </button>
  );
}

function AppUpdateBanner({
  update,
  onRestart,
  onDismiss
}: {
  update: AppUpdate;
  onRestart: () => void;
  onDismiss: () => void;
}) {
  if (update.downloading) {
    return (
      <div className="fixed bottom-4 right-4 bg-zinc-900 border border-zinc-700 rounded-lg shadow-xl p-4 flex items-center gap-3 z-50">
        <svg className="w-5 h-5 text-blue-400 animate-spin" fill="none" viewBox="0 0 24 24">
          <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
          <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
        </svg>
        <div>
          <div className="text-sm font-medium text-white">Downloading update...</div>
          <div className="text-xs text-zinc-400">v{update.version} ({Math.round(update.progress)}%)</div>
        </div>
      </div>
    );
  }

  if (update.downloadedAndReady) {
    return (
      <div className="fixed bottom-4 right-4 bg-zinc-900 border border-blue-500/50 rounded-lg shadow-xl p-4 z-50">
        <div className="flex items-start gap-3">
          <div className="w-8 h-8 bg-blue-500/20 rounded-full flex items-center justify-center flex-shrink-0">
            <svg className="w-4 h-4 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
            </svg>
          </div>
          <div className="flex-1">
            <div className="text-sm font-medium text-white">Update ready to install</div>
            <div className="text-xs text-zinc-400 mt-0.5">Version {update.version} has been downloaded</div>
            <div className="flex items-center gap-2 mt-3">
              <button
                onClick={onRestart}
                className="px-3 py-1.5 bg-blue-600 hover:bg-blue-500 text-white text-xs font-medium rounded-md transition-colors"
              >
                Restart Now
              </button>
              <button
                onClick={onDismiss}
                className="px-3 py-1.5 text-zinc-400 hover:text-white text-xs font-medium transition-colors"
              >
                Later
              </button>
            </div>
          </div>
        </div>
      </div>
    );
  }

  return null;
}

function LocalToolCard({
  status,
  isLoading,
  isUpdating,
  onRefresh,
  onUpdate,
  onLaunch
}: {
  status: ToolStatus | null;
  isLoading: boolean;
  isUpdating: boolean;
  onRefresh: () => void;
  onUpdate: () => void;
  onLaunch: () => void;
}) {
  const shortCommit = (hash: string | null) => hash?.slice(0, 7) || "—";

  return (
    <div className="bg-zinc-900 rounded-xl border border-zinc-800 overflow-hidden">
      <div className="p-5 flex items-start gap-4">
        <div className="w-14 h-14 bg-gradient-to-br from-purple-600 to-blue-600 rounded-xl flex items-center justify-center flex-shrink-0">
          <svg className="w-7 h-7 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
        </div>

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <h3 className="text-lg font-semibold text-white">Resolve Sync Script</h3>
            <span className="px-1.5 py-0.5 bg-zinc-800 text-zinc-400 text-xs rounded font-medium">
              Local
            </span>
          </div>
          <p className="text-sm text-zinc-400 mt-0.5">Auto-imports files to DaVinci Resolve</p>

          {isLoading ? (
            <div className="mt-3 flex items-center gap-2 text-zinc-500">
              <svg className="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
              </svg>
              <span className="text-sm">Checking for updates...</span>
            </div>
          ) : status ? (
            <div className="mt-3 space-y-1.5">
              {status.installed ? (
                <>
                  <div className="flex items-center gap-2">
                    <span className="text-xs text-zinc-500 w-16">Version:</span>
                    <span className="text-sm text-zinc-300">{status.local_version || "—"}</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="text-xs text-zinc-500 w-16">Local:</span>
                    <code className="text-sm text-zinc-400 font-mono">{shortCommit(status.local_commit)}</code>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="text-xs text-zinc-500 w-16">Remote:</span>
                    <code className="text-sm text-zinc-400 font-mono">{shortCommit(status.remote_commit)}</code>
                    {status.has_update && (
                      <span className="px-1.5 py-0.5 bg-amber-500/20 text-amber-400 text-xs rounded font-medium">
                        Update available
                      </span>
                    )}
                  </div>
                </>
              ) : (
                <div className="flex items-center gap-2 text-zinc-500">
                  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                  </svg>
                  <span className="text-sm">Not installed</span>
                </div>
              )}
            </div>
          ) : null}
        </div>

        <button
          onClick={onRefresh}
          disabled={isLoading}
          className="p-2 text-zinc-500 hover:text-white hover:bg-zinc-800 rounded-lg transition-colors disabled:opacity-50"
          title="Refresh status"
        >
          <svg className={`w-4 h-4 ${isLoading ? 'animate-spin' : ''}`} fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
        </button>
      </div>

      <div className="px-5 py-3 bg-zinc-950/50 border-t border-zinc-800 flex items-center gap-2">
        {status?.installed && (
          <>
            <button
              onClick={onLaunch}
              className="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white text-sm font-medium rounded-lg transition-colors"
            >
              Open
            </button>

            {status.has_update && (
              <button
                onClick={onUpdate}
                disabled={isUpdating}
                className="px-4 py-2 bg-zinc-800 hover:bg-zinc-700 text-white text-sm font-medium rounded-lg transition-colors disabled:opacity-50 flex items-center gap-2"
              >
                {isUpdating ? (
                  <>
                    <svg className="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                      <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                      <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                    </svg>
                    Updating...
                  </>
                ) : (
                  "Update"
                )}
              </button>
            )}
          </>
        )}
      </div>
    </div>
  );
}

function WebAppCard({ app, onOpen }: { app: WebApp; onOpen: () => void }) {
  return (
    <div className="bg-zinc-900 rounded-xl border border-zinc-800 overflow-hidden">
      <div className="p-5 flex items-start gap-4">
        <div className={`w-14 h-14 ${app.gradient} rounded-xl flex items-center justify-center flex-shrink-0`}>
          {app.icon}
        </div>

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <h3 className="text-lg font-semibold text-white">{app.name}</h3>
            <span className="px-1.5 py-0.5 bg-sky-500/20 text-sky-400 text-xs rounded font-medium flex items-center gap-1">
              <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />
              </svg>
              Web
            </span>
          </div>
          <p className="text-sm text-zinc-400 mt-0.5">{app.description}</p>
          <p className="text-xs text-zinc-600 mt-2 font-mono">{app.url}</p>
        </div>
      </div>

      <div className="px-5 py-3 bg-zinc-950/50 border-t border-zinc-800 flex items-center gap-2">
        <button
          onClick={onOpen}
          className="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white text-sm font-medium rounded-lg transition-colors flex items-center gap-2"
        >
          Open
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
          </svg>
        </button>
      </div>
    </div>
  );
}

const webApps: WebApp[] = [
  {
    name: "Spellbook",
    description: "Story production management platform",
    url: "https://spellbook.story.inc",
    gradient: "bg-gradient-to-br from-amber-500 to-orange-600",
    icon: (
      <svg className="w-7 h-7 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
      </svg>
    ),
  },
  {
    name: "Story Portal",
    description: "Team collaboration and resources hub",
    url: "https://portal.story.inc",
    gradient: "bg-gradient-to-br from-emerald-500 to-teal-600",
    icon: (
      <svg className="w-7 h-7 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
      </svg>
    ),
  },
];

function AppsPage({
  status,
  isLoading,
  isUpdating,
  message,
  onRefresh,
  onUpdate,
  onLaunch,
  onMessage
}: {
  status: ToolStatus | null;
  isLoading: boolean;
  isUpdating: boolean;
  message: { type: 'success' | 'error'; text: string } | null;
  onRefresh: () => void;
  onUpdate: () => void;
  onLaunch: () => void;
  onMessage: (msg: { type: 'success' | 'error'; text: string } | null) => void;
}) {
  const handleOpenWebApp = async (url: string) => {
    try {
      await openUrl(url);
    } catch (err) {
      onMessage({ type: 'error', text: `Failed to open: ${err}` });
    }
  };

  return (
    <div className="flex-1 bg-zinc-950 overflow-auto">
      <div className="p-8">
        <div className="flex items-center justify-between mb-6">
          <h1 className="text-2xl font-bold text-white">Apps</h1>
        </div>

        {message && (
          <div className={`mb-4 px-4 py-3 rounded-lg max-w-2xl ${
            message.type === 'success'
              ? 'bg-green-500/20 text-green-400 border border-green-500/30'
              : 'bg-red-500/20 text-red-400 border border-red-500/30'
          }`}>
            {message.text}
          </div>
        )}

        <div className="max-w-2xl space-y-4">
          <div className="mb-6">
            <h2 className="text-sm font-medium text-zinc-500 uppercase tracking-wider mb-3">Local Tools</h2>
            <LocalToolCard
              status={status}
              isLoading={isLoading}
              isUpdating={isUpdating}
              onRefresh={onRefresh}
              onUpdate={onUpdate}
              onLaunch={onLaunch}
            />
          </div>

          <div>
            <h2 className="text-sm font-medium text-zinc-500 uppercase tracking-wider mb-3">Web Apps</h2>
            <div className="space-y-4">
              {webApps.map((app) => (
                <WebAppCard
                  key={app.name}
                  app={app}
                  onOpen={() => handleOpenWebApp(app.url)}
                />
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function SettingsPage({
  settings,
  onSettingsChange
}: {
  settings: Settings;
  onSettingsChange: (key: keyof Settings, value: boolean) => void;
}) {
  return (
    <div className="flex-1 bg-zinc-950 overflow-auto">
      <div className="p-8">
        <h1 className="text-2xl font-bold text-white mb-6">Settings</h1>

        <div className="max-w-2xl space-y-6">
          <div className="bg-zinc-900 rounded-xl border border-zinc-800 p-5">
            <h2 className="text-lg font-semibold text-white mb-4">General</h2>
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-sm font-medium text-white">Auto-update tools on launch</div>
                  <div className="text-xs text-zinc-500">Automatically update local tools when updates are available</div>
                </div>
                <Toggle
                  enabled={settings.autoUpdateOnLaunch}
                  onChange={(value) => onSettingsChange('autoUpdateOnLaunch', value)}
                />
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <div className="text-sm font-medium text-white">Launch at login</div>
                  <div className="text-xs text-zinc-500">Automatically open Story Launcher when you log in</div>
                </div>
                <Toggle
                  enabled={settings.launchAtLogin}
                  onChange={(value) => onSettingsChange('launchAtLogin', value)}
                />
              </div>
            </div>
          </div>

          <div className="bg-zinc-900 rounded-xl border border-zinc-800 p-5">
            <h2 className="text-lg font-semibold text-white mb-4">About</h2>
            <div className="text-sm text-zinc-400 space-y-2">
              <div className="flex justify-between">
                <span>Version</span>
                <span className="text-zinc-300">{APP_VERSION}</span>
              </div>
              <div className="flex justify-between">
                <span>Build</span>
                <span className="text-zinc-300">Tauri + React</span>
              </div>
              <div className="pt-2 border-t border-zinc-800 mt-3">
                <p className="text-zinc-500 text-xs">© 2024 Story Co. All rights reserved.</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function App() {
  const [currentPage, setCurrentPage] = useState<Page>("apps");
  const [status, setStatus] = useState<ToolStatus | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isUpdating, setIsUpdating] = useState(false);
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);
  const [settings, setSettings] = useState<Settings>(defaultSettings);
  const [settingsLoaded, setSettingsLoaded] = useState(false);
  const [appUpdate, setAppUpdate] = useState<AppUpdate | null>(null);
  const [updateDismissed, setUpdateDismissed] = useState(false);

  // Load settings from store
  useEffect(() => {
    const loadSettings = async () => {
      try {
        const store = await load(STORE_NAME);
        const autoUpdate = await store.get<boolean>('autoUpdateOnLaunch');
        const launchAtLogin = await store.get<boolean>('launchAtLogin');

        setSettings({
          autoUpdateOnLaunch: autoUpdate ?? defaultSettings.autoUpdateOnLaunch,
          launchAtLogin: launchAtLogin ?? defaultSettings.launchAtLogin,
        });

        // Sync autostart state with stored setting
        const autostartEnabled = await isEnabled();
        if (launchAtLogin !== undefined && autostartEnabled !== launchAtLogin) {
          if (launchAtLogin) {
            await enable();
          } else {
            await disable();
          }
        }
      } catch (err) {
        console.error('Failed to load settings:', err);
      } finally {
        setSettingsLoaded(true);
      }
    };
    loadSettings();
  }, []);

  // Handle settings changes
  const handleSettingsChange = async (key: keyof Settings, value: boolean) => {
    setSettings(prev => ({ ...prev, [key]: value }));

    try {
      const store = await load(STORE_NAME);
      await store.set(key, value);
      await store.save();

      // Handle launch at login toggle
      if (key === 'launchAtLogin') {
        if (value) {
          await enable();
        } else {
          await disable();
        }
      }
    } catch (err) {
      console.error('Failed to save setting:', err);
    }
  };

  const checkStatus = useCallback(async () => {
    setIsLoading(true);
    try {
      const result = await invoke<ToolStatus>("check_tool_status");
      setStatus(result);
      return result;
    } catch (err) {
      console.error("Failed to check status:", err);
      return null;
    } finally {
      setIsLoading(false);
    }
  }, []);

  const handleUpdate = useCallback(async () => {
    setIsUpdating(true);
    setMessage(null);
    try {
      const result = await invoke<UpdateResult>("update_tool");
      if (result.success) {
        setMessage({ type: 'success', text: 'Update complete!' });
        await checkStatus();
      } else {
        setMessage({ type: 'error', text: result.message });
      }
      return result;
    } catch (err) {
      setMessage({ type: 'error', text: String(err) });
      return { success: false, message: String(err) };
    } finally {
      setIsUpdating(false);
    }
  }, [checkStatus]);

  const handleLaunch = async () => {
    try {
      const result = await invoke<UpdateResult>("launch_tool");
      if (!result.success) {
        setMessage({ type: 'error', text: result.message });
      }
    } catch (err) {
      setMessage({ type: 'error', text: String(err) });
    }
  };

  // Auto-check and auto-update on app launch
  useEffect(() => {
    if (!settingsLoaded) return;

    const initializeApp = async () => {
      const toolStatus = await checkStatus();

      // If auto-update is enabled and there's an update available, run it silently
      if (settings.autoUpdateOnLaunch && toolStatus?.has_update) {
        setIsUpdating(true);
        try {
          await invoke<UpdateResult>("update_tool");
          // Refresh status after update
          await checkStatus();
        } catch (err) {
          console.error("Auto-update failed:", err);
        } finally {
          setIsUpdating(false);
        }
      }
    };

    initializeApp();
  }, [settingsLoaded, settings.autoUpdateOnLaunch, checkStatus]);

  // Update tray icon when status changes
  useEffect(() => {
    const hasUpdate = status?.has_update ?? false;
    invoke("set_tray_update_icon", { hasUpdate }).catch(console.error);
  }, [status?.has_update]);

  // Listen for tray menu "Check for Updates" event
  useEffect(() => {
    const unlisten = listen("check-updates", () => {
      checkStatus();
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [checkStatus]);

  // Check for app updates on launch
  useEffect(() => {
    const checkForAppUpdate = async () => {
      try {
        const update = await check();
        if (update) {
          setAppUpdate({
            version: update.version,
            downloadedAndReady: false,
            downloading: true,
            progress: 0,
          });

          // Download the update
          let downloaded = 0;
          let contentLength = 0;

          await update.downloadAndInstall((event) => {
            switch (event.event) {
              case 'Started':
                contentLength = (event.data as { contentLength?: number }).contentLength || 0;
                setAppUpdate(prev => prev ? { ...prev, downloading: true, progress: 0 } : null);
                break;
              case 'Progress':
                downloaded += (event.data as { chunkLength: number }).chunkLength;
                const progress = contentLength > 0 ? (downloaded / contentLength) * 100 : 50;
                setAppUpdate(prev => prev ? { ...prev, progress: Math.min(progress, 99) } : null);
                break;
              case 'Finished':
                setAppUpdate(prev => prev ? { ...prev, downloading: false, downloadedAndReady: true, progress: 100 } : null);
                break;
            }
          });
        }
      } catch (err) {
        console.error('Failed to check for app updates:', err);
        setAppUpdate(null);
      }
    };

    // Delay the update check slightly to not interfere with startup
    const timer = setTimeout(checkForAppUpdate, 3000);
    return () => clearTimeout(timer);
  }, []);

  // Handle app restart for update
  const handleAppRestart = async () => {
    try {
      await relaunch();
    } catch (err) {
      console.error('Failed to restart app:', err);
    }
  };

  // Calculate update count for badge
  const updateCount = status?.has_update ? 1 : 0;

  return (
    <div className="h-screen flex bg-zinc-950 text-white">
      <Sidebar
        currentPage={currentPage}
        onPageChange={setCurrentPage}
        updateCount={updateCount}
      />
      {currentPage === "apps" ? (
        <AppsPage
          status={status}
          isLoading={isLoading}
          isUpdating={isUpdating}
          message={message}
          onRefresh={checkStatus}
          onUpdate={handleUpdate}
          onLaunch={handleLaunch}
          onMessage={setMessage}
        />
      ) : (
        <SettingsPage
          settings={settings}
          onSettingsChange={handleSettingsChange}
        />
      )}

      {appUpdate && !updateDismissed && (
        <AppUpdateBanner
          update={appUpdate}
          onRestart={handleAppRestart}
          onDismiss={() => setUpdateDismissed(true)}
        />
      )}
    </div>
  );
}

export default App;
