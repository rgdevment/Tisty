import { useState } from "react";
import type { Choice } from "./Menu";
import PictureViewer from "./PictureViewer";
import { pictureChoices, type Shot, shotsOf } from "./paging";

type Ways = { readonly save?: (kept: string) => void; readonly open?: (kept: string) => void };

// The pictures of a page looked at large: what a click opens, what their menu offers, and the viewer itself.
export const usePictures = ({ save, open }: Ways) => {
  const [viewing, setViewing] = useState<{ shots: Shot[]; nth: number } | null>(null);
  const view = (shots: Shot[], nth: number) => setViewing({ shots, nth });
  const offered = (page: Document | undefined, picture: number | null): Choice[] => {
    const shots = picture !== null && page ? shotsOf(page) : [];
    return pictureChoices(shots, picture, {
      look: (nth) => view(shots, nth),
      save: (kept) => save?.(kept),
      open: (kept) => open?.(kept),
    });
  };
  const viewer = viewing && (
    <PictureViewer
      shots={viewing.shots}
      start={viewing.nth}
      onSave={save}
      onOpen={open}
      onClose={() => setViewing(null)}
    />
  );
  return { view, offered, viewer };
};
