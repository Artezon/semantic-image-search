<template>
  <BaseModal :title="title" :is-top="isTop" @close="$emit('close')">
    <p v-html="message" />
    <template #footer>
      <button class="btn full-width-btn" autofocus @click="$emit('close')">OK</button>
    </template>
  </BaseModal>
</template>

<script setup lang="ts">
import BaseModal from "./BaseModal.vue";

defineProps<{
  title?: string | null;
  message: string;
  isTop?: boolean;
}>();

defineEmits(["close"]);
</script>
