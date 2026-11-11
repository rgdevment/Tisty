import { type List, listArchive } from "../core";
import { dropList, settleList } from "../listMenus";
import { t } from "../locales";
import Naming from "./Naming";

interface Props {
  list: List;
  held: number;
  onClose: () => void;
  onChanged: () => void;
  onGone?: (list: string) => void;
  onError: (problem: unknown) => void;
}

export default function ListNaming({ list, held, onClose, onChanged, onGone, onError }: Props) {
  const done = () => {
    onClose();
    onChanged();
  };
  const away = () => {
    onClose();
    if (onGone) onGone(list.id);
    else onChanged();
  };
  const failed = (problem: unknown) => {
    onChanged();
    onError(problem);
  };

  return (
    <Naming
      title={t("editList")}
      invite={t("listName")}
      called={list.name}
      drawn={list.icon ?? undefined}
      painted={list.color ?? undefined}
      action={t("saveIt")}
      dropWord={t("listDrop")}
      onName={(called, drawn, colour) =>
        settleList(list, called, drawn, colour).then(done).catch(failed)
      }
      onDrop={() =>
        dropList(list)
          .then((dropped) => dropped && away())
          .catch(onError)
      }
      archiveWord={t("listArchive")}
      onArchive={held === 0 ? () => listArchive(list.id).then(away).catch(onError) : undefined}
      onClose={onClose}
    />
  );
}
