import { invoke } from "@tauri-apps/api/core";
import type { ThemeMode } from "../types";

const STORAGE_KEY = "theme";

let themeMode = $state<ThemeMode>("system");
let mediaQuery: MediaQueryList | null = null;

function applyTheme(mode: ThemeMode) {
  const isDark =
    mode === "dark" ||
    (mode === "system" &&
      window.matchMedia("(prefers-color-scheme: dark)").matches);

  document.documentElement.classList.toggle("dark", isDark);
  document.documentElement.style.colorScheme = isDark ? "dark" : "light";
}

function onSystemThemeChange() {
  if (themeMode === "system") {
    applyTheme("system");
  }
}

function startWatchingSystem() {
  if (mediaQuery) return;
  mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
  mediaQuery.addEventListener("change", onSystemThemeChange);
}

function stopWatchingSystem() {
  if (!mediaQuery) return;
  mediaQuery.removeEventListener("change", onSystemThemeChange);
  mediaQuery = null;
}

export function getThemeMode(): ThemeMode {
  return themeMode;
}

export async function initTheme() {
  try {
    const mode = (await invoke<string>("get_theme")) as ThemeMode;
    themeMode = mode;
    localStorage.setItem(STORAGE_KEY, mode);
    applyTheme(mode);
  } catch {
    themeMode = "system";
    applyTheme("system");
  }
  startWatchingSystem();
}

export async function setThemeMode(mode: ThemeMode) {
  await invoke("set_theme", { mode });
  themeMode = mode;
  localStorage.setItem(STORAGE_KEY, mode);
  applyTheme(mode);

  if (mode === "system") {
    startWatchingSystem();
  } else {
    stopWatchingSystem();
  }
}
