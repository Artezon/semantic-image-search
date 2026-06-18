import { openModal } from "../../composables/useModal";
import InfoModal from "./InfoModal.vue";
import ConfirmRemoveFolderModal from "./ConfirmRemoveFolderModal.vue";

export async function showInfoModal(message: string, title?: string) {
  await openModal(InfoModal, { message, title: title ?? null });
}

export async function showConfirmRemoveFolderModal(folderPath: string) {
  await openModal(ConfirmRemoveFolderModal, { folderPath });
}
