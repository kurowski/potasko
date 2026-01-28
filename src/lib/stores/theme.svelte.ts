/**
 * Theme store for theme selection and light/dark mode toggle
 * with system preference detection and localStorage persistence.
 */

const THEME_KEY = 'potasko-theme';
const MODE_KEY = 'potasko-mode';

export type ThemeMode = 'light' | 'dark' | 'system';

// Available Skeleton themes (keep in sync with CSS imports in app.css)
export const AVAILABLE_THEMES = [
  { value: 'catppuccin', label: 'Catppuccin' },
  { value: 'cerberus', label: 'Cerberus' },
  { value: 'concord', label: 'Concord' },
  { value: 'crimson', label: 'Crimson' },
  { value: 'fennec', label: 'Fennec' },
  { value: 'hamlindigo', label: 'Hamlindigo' },
  { value: 'legacy', label: 'Legacy' },
  { value: 'mint', label: 'Mint' },
  { value: 'modern', label: 'Modern' },
  { value: 'mona', label: 'Mona' },
  { value: 'nosh', label: 'Nosh' },
  { value: 'nouveau', label: 'Nouveau' },
  { value: 'pine', label: 'Pine' },
  { value: 'reign', label: 'Reign' },
  { value: 'rocket', label: 'Rocket' },
  { value: 'rose', label: 'Rose' },
  { value: 'sahara', label: 'Sahara' },
  { value: 'seafoam', label: 'Seafoam' },
  { value: 'terminus', label: 'Terminus' },
  { value: 'vintage', label: 'Vintage' },
  { value: 'vox', label: 'Vox' },
  { value: 'wintry', label: 'Wintry' },
] as const;

export type ThemeName = typeof AVAILABLE_THEMES[number]['value'];

const DEFAULT_THEME: ThemeName = 'cerberus';

class ThemeStore {
  theme = $state<ThemeName>(DEFAULT_THEME);
  mode = $state<ThemeMode>('system');
  resolvedMode = $state<'light' | 'dark'>('light');
  private initialized = false;

  /** Initialize the store - must be called from onMount in a component */
  init() {
    if (this.initialized || typeof window === 'undefined') return;
    this.initialized = true;

    // Load saved preferences
    const savedTheme = localStorage.getItem(THEME_KEY) as ThemeName | null;
    if (savedTheme && AVAILABLE_THEMES.some(t => t.value === savedTheme)) {
      this.theme = savedTheme;
    }

    const savedMode = localStorage.getItem(MODE_KEY) as ThemeMode | null;
    if (savedMode && ['light', 'dark', 'system'].includes(savedMode)) {
      this.mode = savedMode;
    }

    // Apply theme and mode
    this.applyTheme();
    this.updateResolvedMode();

    // Listen for system preference changes
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    mediaQuery.addEventListener('change', () => {
      if (this.mode === 'system') {
        this.updateResolvedMode();
      }
    });
  }

  private updateResolvedMode() {
    if (typeof window === 'undefined') return;

    let resolved: 'light' | 'dark';
    if (this.mode === 'system') {
      resolved = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    } else {
      resolved = this.mode;
    }

    this.resolvedMode = resolved;
    this.applyMode(resolved);
  }

  private applyMode(mode: 'light' | 'dark') {
    if (typeof document === 'undefined') return;
    // Use .dark class (Tailwind selector strategy)
    if (mode === 'dark') {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  }

  private applyTheme() {
    if (typeof document === 'undefined') return;
    document.documentElement.setAttribute('data-theme', this.theme);
  }

  setTheme(theme: ThemeName) {
    this.theme = theme;
    localStorage.setItem(THEME_KEY, theme);
    this.applyTheme();
  }

  setMode(mode: ThemeMode) {
    this.mode = mode;
    localStorage.setItem(MODE_KEY, mode);
    this.updateResolvedMode();
  }

  get isDark() {
    return this.resolvedMode === 'dark';
  }
}

export const themeStore = new ThemeStore();
