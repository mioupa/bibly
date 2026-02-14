export const FONT_NAME_STORAGE_KEY = 'bibly_font_name';
const LEGACY_FONT_PRESET_STORAGE_KEY = 'bibly_font_preset';

const DEFAULT_FONT_STACK = '-apple-system, BlinkMacSystemFont, "Hiragino Sans", "Yu Gothic UI", "Yu Gothic", "Meiryo", "Noto Sans JP", "Segoe UI", sans-serif';

const LEGACY_PRESET_TO_FONT_NAME: Record<string, string> = {
  system: '',
  gothic: 'Hiragino Kaku Gothic ProN',
  mincho: 'Hiragino Mincho ProN',
  rounded: 'Hiragino Maru Gothic ProN',
};

function normalizeFontName(fontName: string): string {
  return fontName.trim().replace(/^['\"]+|['\"]+$/g, '');
}

function escapeCssString(value: string): string {
  return value.replace(/\\/g, '\\\\').replace(/"/g, '\\"');
}

function buildFontFamily(fontName: string): string {
  const normalized = normalizeFontName(fontName);
  if (!normalized) {
    return DEFAULT_FONT_STACK;
  }
  return `"${escapeCssString(normalized)}", ${DEFAULT_FONT_STACK}`;
}

export function getSavedFontName(): string {
  const saved = localStorage.getItem(FONT_NAME_STORAGE_KEY);
  if (saved != null) {
    return normalizeFontName(saved);
  }

  const legacyPreset = localStorage.getItem(LEGACY_FONT_PRESET_STORAGE_KEY);
  if (legacyPreset && legacyPreset in LEGACY_PRESET_TO_FONT_NAME) {
    return LEGACY_PRESET_TO_FONT_NAME[legacyPreset];
  }

  return '';
}

export function applyFontName(fontName: string): void {
  document.documentElement.style.setProperty('--app-font-family', buildFontFamily(fontName));
}

export function saveAndApplyFontName(fontName: string): void {
  const normalized = normalizeFontName(fontName);
  localStorage.setItem(FONT_NAME_STORAGE_KEY, normalized);
  applyFontName(normalized);
}
