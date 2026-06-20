<template>
  <BaseModal
    :title="title"
    width="460px"
    :dismissible="!removing"
    :is-top="isTop"
    @close="onCancel"
  >
    <template v-if="!removing">
      <p v-html="$t('confirm_remove_folder.modal.message', { path: folderPath })" />
      <p>{{ $t("confirm_remove_folder.modal.description") }}</p>
    </template>
    <div v-else class="spinner-wrapper">
      <div class="spinner" />
    </div>
    <template #footer>
      <button v-if="!removing" class="btn full-width-btn" @click="onCancel">
        {{ $t("action.no") }}
      </button>
      <button v-if="!removing" class="btn full-width-btn primary" autofocus @click="onConfirm">
        {{ $t("action.yes") }}
      </button>
    </template>
  </BaseModal>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import BaseModal from "./BaseModal.vue";
import { indexingState } from "../../store";

const { t } = useI18n({ useScope: "global" });

const props = defineProps<{
  folderPath: string;
  isTop?: boolean;
}>();

const emit = defineEmits<{ close: [] }>();

const removing = ref(false);
const title = ref<string | null>(t("confirm_remove_folder.modal.title"));

async function onConfirm() {
  removing.value = true;
  title.value = null;
  try {
    if (indexingState.value === "indexing" || indexingState.value === "preparing") {
      indexingState.value = "pausing";
      await invoke("pause_indexing");
    }
    await invoke("remove_directory", { path: props.folderPath });
  } finally {
    emit("close");
  }
}

function onCancel() {
  if (!removing.value) emit("close");
}
</script>

<style scoped>
.spinner-wrapper {
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 32px;
}
</style>
