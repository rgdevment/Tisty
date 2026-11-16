import type { EditorView } from "@codemirror/view";
import {
  chooseOrLet,
  doubled,
  droppedBlock,
  FORMATS,
  moving,
  pressed,
  wrapped,
} from "tisty-editor/edit";
import { copied } from "../core";
import { t } from "../locales";
import type { Choice } from "./Menu";

export const ALT = typeof navigator !== "undefined" && /Mac/.test(navigator.platform) ? "⌥" : "Alt";

export const livelyChoices = (editing: EditorView): Choice[] => {
  const { from, to } = editing.state.selection.main;
  const picked = from !== to;
  const chosen = editing.state.sliceDoc(from, to);
  const back = () => editing.focus();
  const run = (act: (view: EditorView) => unknown) => () => {
    act(editing);
    back();
  };
  const writing: Choice[] = picked
    ? [
        {
          key: "cut",
          label: t("cutIt"),
          keys: pressed("X"),
          onPick: () => {
            const was = editing.state.doc;
            void copied(chosen).then(() => {
              if (editing.state.doc === was)
                editing.dispatch({ changes: { from, to }, userEvent: "delete.cut" });
              back();
            });
          },
        },
        {
          key: "copy",
          label: t("copyIt"),
          keys: pressed("C"),
          onPick: () => void copied(chosen).then(back),
        },
        ...FORMATS.map(([said, keys, mark, close], at) => ({
          key: `format-${said}`,
          label: t(said),
          keys: pressed(keys),
          apart: at === 0,
          onPick: () => {
            wrapped(editing, mark, close);
            back();
          },
        })),
      ]
    : [];
  return [
    ...writing,
    {
      key: "block-choose",
      label: t("blockChoose"),
      keys: "Esc",
      apart: picked,
      onPick: run(chooseOrLet),
    },
    { key: "block-double", label: t("blockDouble"), keys: pressed("D"), onPick: run(doubled) },
    { key: "block-up", label: t("blockUp"), keys: `${ALT} ↑`, onPick: run(moving(-1)) },
    { key: "block-down", label: t("blockDown"), keys: `${ALT} ↓`, onPick: run(moving(1)) },
    {
      key: "block-drop",
      label: t("blockDrop"),
      danger: true,
      apart: true,
      onPick: run(droppedBlock),
    },
  ];
};
