import { reactive, watch } from "vue";

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
