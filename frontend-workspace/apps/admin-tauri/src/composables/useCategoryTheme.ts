export type ThemeKey = "cyan" | "orange" | "purple" | "emerald" | "blue" | "red" | "amber" | "silver";

export interface ThemeDef {
  hex: string;
  label: string;
}

export const THEME_DICT: Record<ThemeKey, ThemeDef> = {
  cyan:    { hex: "#00e5ff", label: "极客青" },
  orange:  { hex: "#ff6b35", label: "废土橙" },
  purple:  { hex: "#b44dff", label: "冷核紫" },
  emerald: { hex: "#39ff14", label: "脉冲绿" },
  blue:    { hex: "#4169e1", label: "深空蓝" },
  red:     { hex: "#ff2d2d", label: "熔岩红" },
  amber:   { hex: "#ffbe0b", label: "琥珀黄" },
  silver:  { hex: "#a0a0a0", label: "钛合金银" },
};

export const THEME_KEYS = Object.keys(THEME_DICT) as ThemeKey[];

export function hexForKey(key: string | null | undefined): string | null {
  if (!key) return null;
  return (THEME_DICT as Record<string, ThemeDef>)[key]?.hex ?? null;
}

export function toRgba(hex: string, alpha: number): string {
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  return `rgba(${r},${g},${b},${alpha})`;
}
