import { i18n } from "./i18n";

const { t } = i18n.global;

// Enforce min/max on number inputs
export function clamp(val: number, min: number, max: number, def?: number) {
  if (!val && val !== 0) return def ?? val;
  return Math.min(Math.max(val, min), max);
}

export function formatSeconds(totalSeconds: number): string {
  const h = Math.floor(totalSeconds / 3600);
  const rem = totalSeconds % 3600;
  const m = Math.floor(rem / 60);
  const s = rem % 60;

  const parts: string[] = [];
  if (h > 0) parts.push(t("duration.hours", { h }));
  if (m > 0) parts.push(t("duration.minutes", { m }));
  parts.push(t("duration.seconds", { s }));
  return parts.join(" ");
}
