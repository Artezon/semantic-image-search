import { reactive, watch } from "vue";
import i18n from "./i18n";

export function numericSetting(min: number, max: number, def: number, step?: number) {
  const state = reactive({ value: def, min, max, step, default: def });
  watch(
    () => state.value,
    (v) => (state.value = clamp(v, min, max, def)),
  );
  return state;
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
