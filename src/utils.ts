import i18n from "./i18n";

export function sanitizeNumberInput(event: Event, min: number, max: number, prev: number): number {
  const input = event.target as HTMLInputElement;
  const parsed = Number(input.value);
  const result = input.value.trim() !== "" && !isNaN(parsed) ? clamp(parsed, min, max) : prev;

  // Force the DOM value back, in case the ref didn't change
  input.value = String(result);
  return result;
}

// Enforce min/max on number inputs
function clamp(val: number, min: number, max: number, def?: number) {
  if (!val && val !== 0) return def ?? val;
  return Math.min(Math.max(val, min), max);
}

export function formatSeconds(totalSeconds: number): string {
  const { t } = i18n.global;
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
