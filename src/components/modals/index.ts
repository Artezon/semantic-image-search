import { openModal } from "../../composables/useModal";
import InfoModal from "./InfoModal.vue";

export async function showInfoModal(message: string, title?: string) {
  await openModal(InfoModal, { message, title: title ?? null });
}
