import { ref, markRaw } from "vue";
import type { Component } from "vue";

interface ModalEntry {
  component: Component;
  props: Record<string, unknown>;
  resolve: (result: unknown) => void;
}

const modalStack = ref<ModalEntry[]>([]);

export function openModal<T = unknown>(
  component: Component,
  props: Record<string, unknown> = {},
): Promise<T> {
  return new Promise((resolve) => {
    modalStack.value.push({
      component: markRaw(component),
      props,
      resolve: resolve as (result: unknown) => void,
    });
  });
}

export function closeModal(result: unknown = null) {
  const modal = modalStack.value.pop();
  modal?.resolve(result);
}

export function useModalStack() {
  return { modalStack, closeModal };
}
