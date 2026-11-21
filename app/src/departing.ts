import { noteTrouble, parted, partingStays, partingWaits } from "./core";
import { t } from "./locales";
import { leftUnsaved, unsettled } from "./saving";
import { sure } from "./sure";

const SHOWN_AFTER_MS = 1000;

const asked = async (): Promise<void> => {
  const asking = leftUnsaved();
  let shown: ReturnType<typeof setTimeout> | undefined;
  if (unsettled()) {
    void partingWaits(false).catch(() => {});
    shown = setTimeout(() => void partingWaits(true).catch(() => {}), SHOWN_AFTER_MS);
  }
  const unsaved = await asking.finally(() => clearTimeout(shown));
  if (!unsaved) return parted();
  await partingWaits(true).catch(() => {});
  if (await sure(t("partingUnsaved"))) {
    await noteTrouble("saveStuck").catch(() => {});
    return parted();
  }
  return partingStays();
};

export const departing = (): Promise<void> => asked().catch(() => partingStays().catch(() => {}));
