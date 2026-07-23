export type Theme = "light" | "dark" | "system";
const THEME_KEY = "app-theme";

export function getStoredTheme(): Theme {
  return (localStorage.getItem(THEME_KEY) as Theme) || "system";
}

export function applyTheme(theme: Theme) {
  const resolvedTheme =
    theme === "system"
      ? (window.matchMedia("(prefers-color-scheme: dark)").matches
          ? "dark"
          : "light")
      : theme;

  document.documentElement.setAttribute("data-theme", resolvedTheme);
}

export function setTheme(theme: Theme) {
  localStorage.setItem(THEME_KEY, theme);
  applyTheme(theme);
}