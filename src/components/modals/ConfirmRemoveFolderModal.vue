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
import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import BaseModal from "./BaseModal.vue";
import { indexedDirs, indexedFilesCount, indexingState } from "../../store";
import { pauseIndexing, startOrResumeIndexing } from "../../indexing";

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
  const isStarted = indexingState.value === "indexing" || indexingState.value === "preparing";
  if (isStarted) {
    await pauseIndexing();
    const unwatch = watch(indexingState, async (newState) => {
      if (newState === "paused") {
        unwatch();
        await invoke("remove_directory", { path: props.folderPath });
        indexedDirs.value = await invoke("get_dirs");
        indexedFilesCount.value = await invoke<number>("get_indexed_count");
        emit("close");
        await startOrResumeIndexing();
      }
    });
  } else {
    await invoke("remove_directory", { path: props.folderPath });
    indexedDirs.value = await invoke("get_dirs");
    indexedFilesCount.value = await invoke<number>("get_indexed_count");
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
