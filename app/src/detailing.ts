import {
  addPart,
  type Change,
  complete,
  discard,
  erase,
  fold,
  hang,
  openToAgents,
  patch,
  readAs,
  reopen,
  type Snapshot,
  stillOpen,
  type Task,
  taskDocSettled,
  taskOf,
} from "./core";
import { fill } from "./locales";
import { queued } from "./saving";
import { sure } from "./sure";

export interface Hands {
  data: Snapshot;
  wholes: NonNullable<Snapshot["wholes"]>;
  act: (work: Promise<Task>) => void;
  marking: (id: string, title: string) => void;
  wipe: (task: Task) => void;
  shut: () => void;
  close: () => void;
  openDoc: (paper: string) => void;
  opening: (one: Task) => void;
  say: (words: string) => void;
  fail: (problem: unknown) => void;
}

export const detailOf = (one: Task, h: Hands) => ({
  task: one,
  lists: h.data.every ?? h.data.lists,
  known: h.data.tags.map((tag) => tag.tag),
  onPatch: (change: Change) => h.act(patch(one.id, change)),
  onComplete: () => {
    h.marking(one.id, one.title);
    h.close();
  },
  onDiscard: () => {
    h.act(discard(one.id));
    h.close();
  },
  onReopen: () => h.act(reopen(one.id)),
  onStillOpen: () => h.act(stillOpen(one.id)),
  onErase: () => h.wipe(one),
  onFold: (away: boolean) => h.act(fold(one.id, away)),
  onReadAs: (how: "story" | "trace") => h.act(readAs(one.id, how)),
  onOpenToAgents: (open: boolean) => h.act(openToAgents(one.id, open)),
  onClose: h.shut,
  onError: h.fail,
  onDoc: h.openDoc,
  whole: h.wholes[one.id],
  partOf: one.part_of ? h.wholes[one.part_of]?.title : undefined,
  onAddPart: (title: string) => h.act(addPart(one.id, title).then(() => taskOf(one.id))),
  onOpenPart: (id: string) => taskOf(id).then(h.opening, h.fail),
  onCompletePart: (id: string, title: string) => {
    h.say(fill("saidDone", title));
    h.act(complete(id).then(() => taskOf(one.id)));
  },
  onHang: (whole: string | null) => h.act(hang(one.id, whole)),
  onSettle: (before: string, now: string) =>
    queued(`settle:${one.id}`, () => taskDocSettled(one.id, before, now)).then(
      (changed) => {
        if (changed) h.act(Promise.resolve(changed));
        return null;
      },
      (problem: unknown) => {
        h.fail(problem);
        return (problem as { code?: string } | null)?.code ?? "failed";
      },
    ),
});

export const erasing = (
  task: Task,
  h: { clear: () => void; gone: () => void; fail: (problem: unknown) => void },
) => {
  const entries = task.volume?.journal ?? 0;
  const said = entries
    ? `${fill("eraseSure", task.title)} ${fill("eraseWritten", String(entries))}`
    : fill("eraseSure", task.title);
  sure(said)
    .then((yes) => {
      if (!yes) return;
      h.clear();
      return erase(task.id).then(h.gone);
    })
    .catch(h.fail);
};
