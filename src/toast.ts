import { toast } from "vue-sonner";
import { markRaw, h } from "vue";
import CustomToast from "./components/CustomToast.vue";

export type ToastKind = "info" | "success" | "error";

export type ToastAction = {
  label: string;
  onClick: () => void;
  closeToast: boolean;
};

export function showToast(
  msg: string,
  title?: string,
  kind: ToastKind = "info",
  persistent = false,
  action?: ToastAction,
) {
  const duration = persistent ? Infinity : 5000;
  let id: string | number;
  const close = () => toast.dismiss(id);
  id = toast.custom(
    markRaw({ render: () => h(CustomToast, { msg, title, kind, action, close }) }),
    { duration },
  );
}

export const showInfoToast = (msg: string, title?: string) => showToast(msg, title, "info");
export const showSuccessToast = (msg: string, title?: string) => showToast(msg, title, "success");
export const showErrorToast = (msg: string, title?: string) => showToast(msg, title, "error");
