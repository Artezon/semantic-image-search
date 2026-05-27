<template>
  <input type="number" :value="modelValue" :min="min" :max="max" :step="step" @change="onChange" />
</template>

<script setup lang="ts">
import { clamp } from "../utils";

const props = defineProps<{
  modelValue: number;
  min: number;
  max: number;
  step?: number;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", value: number): void;
}>();

function sanitizeNumberInput(event: Event, min: number, max: number, prev: number): number {
  const input = event.target as HTMLInputElement;
  const parsed = Number(input.value);
  const result = input.value.trim() !== "" && !isNaN(parsed) ? clamp(parsed, min, max) : prev;
  input.value = String(result);
  return result;
}

function onChange(event: Event) {
  emit("update:modelValue", sanitizeNumberInput(event, props.min, props.max, props.modelValue));
}
</script>
