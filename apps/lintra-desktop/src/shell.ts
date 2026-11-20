// Shell model of the desktop UI (T-MIG-P5-01): sections and theme only.
// It holds no validation, report or normative state; those stay in Rust and
// reach later views through the typed IPC client.

/** Sections of the bar ratified by the Tauri brief; Analisi is the initial one. */
export const VIEWS = [
  { id: "analisi", label: "Analisi" },
  { id: "risultati", label: "Risultati" },
  { id: "informazioni", label: "Informazioni" },
] as const;
export type ViewId = (typeof VIEWS)[number]["id"];
export const INITIAL_VIEW: ViewId = "analisi";

/** Theme preference. "system" follows prefers-color-scheme. */
export const THEMES = [
  { id: "system", label: "Sistema" },
  { id: "light", label: "Chiaro" },
  { id: "dark", label: "Scuro" },
] as const;
export type ThemePreference = (typeof THEMES)[number]["id"];
export const INITIAL_THEME: ThemePreference = "system";

/** Narrows untrusted strings (e.g. a form value) to a known section. */
export function isViewId(value: string): value is ViewId {
  return VIEWS.some(view => view.id === value);
}

/** Narrows untrusted strings (e.g. a radio value) to a known theme. */
export function isThemePreference(value: string): value is ThemePreference {
  return THEMES.some(theme => theme.id === value);
}

/**
 * Applies the preference to the root element. Tokens read `data-theme`;
 * removing it lets the media query decide. The choice is kept only for the
 * session: no JS storage is allowed for the adapter (Master Plan §6 delta 2).
 */
export function applyTheme(root: { dataset: DOMStringMap }, preference: ThemePreference): void {
  if (preference === "system") delete root.dataset.theme;
  else root.dataset.theme = preference;
}
