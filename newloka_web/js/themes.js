﻿const THEMES = {
  newloka: {
    "--color-bg": "#f4f8f5",
    "--color-surface": "#ffffff",
    "--color-surface-raised": "#e6f1f5",
    "--color-text": "#4a3c31",
    "--color-text-muted": "#6b5e54",
    "--color-primary": "#2a9d8f",
    "--color-primary-dark": "#1f7a73",
    "--color-success": "#5c8a63",
    "--color-warning": "#d97706",
    "--color-danger": "#c0504d",
    "--color-danger-soft": "#8a3b3b",
    "--color-border": "rgba(42, 157, 143, 0.30)",
    "--color-border-light": "rgba(42, 157, 143, 0.20)",
    "--color-border-subtle": "rgba(42, 157, 143, 0.14)",
    "--color-input-bg": "#ffffff",
    "--color-overlay": "rgba(45, 36, 29, 0.45)",
    "--color-glass-bg": "rgba(255, 255, 255, 0.72)",
    "--color-glass-border": "rgba(255, 255, 255, 0.55)",
    "--color-hover": "rgba(42, 157, 143, 0.08)",
    "--color-active-bg": "rgba(42, 157, 143, 0.12)",
    "--color-row-alt": "rgba(230, 241, 245, 0.5)",
    "--color-chart-bg": "rgba(255,255,255,0.45)",
    "--shadow": "0 4px 24px rgba(0,0,0,0.06)",
    "--header-bg": "linear-gradient(180deg, rgba(244,248,245,0.98), rgba(230,241,245,0.92))",
    "--header-border": "rgba(42, 157, 143, 0.22)",
    "--nav-bg": "#ffffff",
    "--nav-border": "rgba(42, 157, 143, 0.22)",
    "--theme-type": "light",
  },
  dark: {
    "--color-bg": "#0f172a",
    "--color-surface": "#1e293b",
    "--color-surface-raised": "#334155",
    "--color-text": "#f1f5f9",
    "--color-text-muted": "#94a3b8",
    "--color-primary": "#38bdf8",
    "--color-primary-dark": "#0284c7",
    "--color-success": "#22c55e",
    "--color-warning": "#eab308",
    "--color-danger": "#ef4444",
    "--color-danger-soft": "#7f1d1d",
    "--color-border": "rgba(148,163,184,0.12)",
    "--color-border-light": "rgba(148,163,184,0.10)",
    "--color-border-subtle": "rgba(148,163,184,0.08)",
    "--color-input-bg": "#0f172a",
    "--color-overlay": "rgba(0,0,0,0.65)",
    "--color-glass-bg": "rgba(30, 41, 59, 0.85)",
    "--color-glass-border": "rgba(148,163,184,0.15)",
    "--color-hover": "rgba(255,255,255,0.05)",
    "--color-active-bg": "rgba(56,189,248,0.08)",
    "--color-row-alt": "rgba(30,41,59,0.5)",
    "--color-chart-bg": "rgba(255,255,255,0.03)",
    "--shadow": "0 4px 24px rgba(0,0,0,0.35)",
    "--header-bg": "linear-gradient(180deg, rgba(15,23,42,0.98), rgba(15,23,42,0.92))",
    "--header-border": "rgba(148,163,184,0.12)",
    "--nav-bg": "#1e293b",
    "--nav-border": "rgba(148,163,184,0.12)",
    "--theme-type": "dark",
  },
  light: {
    "--color-bg": "#f8fafc",
    "--color-surface": "#ffffff",
    "--color-surface-raised": "#f1f5f9",
    "--color-text": "#0f172a",
    "--color-text-muted": "#64748b",
    "--color-primary": "#0ea5e9",
    "--color-primary-dark": "#0369a1",
    "--color-success": "#22c55e",
    "--color-warning": "#eab308",
    "--color-danger": "#ef4444",
    "--color-danger-soft": "#7f1d1d",
    "--color-border": "rgba(148,163,184,0.22)",
    "--color-border-light": "rgba(148,163,184,0.15)",
    "--color-border-subtle": "rgba(148,163,184,0.10)",
    "--color-input-bg": "#ffffff",
    "--color-overlay": "rgba(15,23,42,0.45)",
    "--color-glass-bg": "rgba(255,255,255,0.72)",
    "--color-glass-border": "rgba(255,255,255,0.55)",
    "--color-hover": "rgba(14,165,233,0.06)",
    "--color-active-bg": "rgba(14,165,233,0.10)",
    "--color-row-alt": "rgba(241,245,249,0.6)",
    "--color-chart-bg": "rgba(255,255,255,0.5)",
    "--shadow": "0 4px 24px rgba(0,0,0,0.08)",
    "--header-bg": "linear-gradient(180deg, rgba(248,250,252,0.98), rgba(248,250,252,0.92))",
    "--header-border": "rgba(148,163,184,0.22)",
    "--nav-bg": "#ffffff",
    "--nav-border": "rgba(148,163,184,0.22)",
    "--theme-type": "light",
  },
};

const CUSTOM_KEYS = [
  "--color-bg",
  "--color-surface",
  "--color-surface-raised",
  "--color-text",
  "--color-text-muted",
  "--color-primary",
  "--color-primary-dark",
  "--color-success",
  "--color-warning",
  "--color-danger",
  "--color-danger-soft",
];

function apply(name, custom = {}) {
  const base = THEMES[name] || THEMES.newloka;
  const vars = name === "custom" ? { ...THEMES.newloka, ...custom } : base;

  const root = document.documentElement;
  for (const [key, value] of Object.entries(vars)) {
    root.style.setProperty(key, value);
  }
  root.setAttribute("data-theme", name);

  // Update meta theme-color for mobile browsers
  const metaTheme = document.querySelector('meta[name="theme-color"]');
  if (metaTheme) {
    metaTheme.content = vars["--color-bg"] || "#fafcfb";
  }

  // Toggle light/dark body class for any legacy selectors
  document.body.classList.toggle("theme-dark", vars["--theme-type"] === "dark");
  document.body.classList.toggle("theme-light", vars["--theme-type"] === "light");
}

function getThemes() {
  return { ...THEMES };
}

function getThemeNames() {
  return Object.keys(THEMES).concat("custom");
}

function getCustomKeys() {
  return [...CUSTOM_KEYS];
}

export { apply, getThemes, getThemeNames, getCustomKeys, THEMES };
