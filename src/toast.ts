import { toast } from "vue-sonner";
import { markRaw, h } from "vue";
import CustomToast from "./components/CustomToast.vue";
import { i18n } from "./i18n";
import type { IndexingResult } from "./types";
import { showInfoModal } from "./components/modals";
import { formatSeconds } from "./utils";

const { t } = i18n.global;

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

export function showIndexingResultToast(result: IndexingResult) {
  const { processed, total, elapsed_secs, errors } = result;

  if (total === 0) {
    showToast(
      t("indexing.complete.toast.up_to_date.msg"),
      t("indexing.complete.toast.up_to_date.header"),
      "info",
      false,
    );
    return;
  }

  let summary = t("indexing.complete.toast.msg", {
    processed,
    total,
    elapsed: formatSeconds(elapsed_secs),
  });

  if (errors.length === 0) {
    showToast(summary, t("indexing.complete.toast.header"), "info");
  } else {
    summary += `\n${t("indexing.complete.toast.errors", { count: errors.length })}`;
    showToast(summary, t("indexing.complete.toast.header"), "info", true, {
      label: t("action.show_errors"),
      onClick: () => {
        const lines = errors.map(([path, err]) => {
          const reason = err.detail ? t(err.detail) : err.code;
          return `${t("error.detail.skipped", { detail: path })}\n${t("error.detail.reason", { detail: reason })}`;
        });
        showInfoModal(
          lines.join("\n\n"),
          t("indexing.complete.modal.errors.header", { count: errors.length }),
        );
      },
      closeToast: true,
    });
  }
}
