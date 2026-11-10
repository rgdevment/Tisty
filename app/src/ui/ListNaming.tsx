import { type List, listArchive } from "../core";
import { dropList, settleList } from "../listMenus";
import { t } from "../locales";
import Naming from "./Naming";

interface Props {
  list: List;
  held: number;
  onClose: () => void;
  onChanged: () => void;
  onDropped?: (list: string) => void;
  onError: (problem: unknown) => void;
}

export default function ListNaming({ list, held, onClose, onChanged, onDropped, onError }: Props) {
  const done = () => {
    onClose();
    onChanged();
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
        settleList(list, called, drawn, colour).then(done).catch(onError)
      }
      onDrop={() =>
        dropList(list)
          .then((gone) => {
            if (!gone) return;
            onClose();
            if (onDropped) onDropped(list.id);
            else onChanged();
          })
          .catch(onError)
      }
      archiveWord={t("listArchive")}
      onArchive={held === 0 ? () => listArchive(list.id).then(done).catch(onError) : undefined}
      onClose={onClose}
    />
  );
}
