import { invoke } from "@tauri-apps/api/core";
import { fill, t } from "./locales";
import { sure } from "./sure";

type Scope = { is: "everyone" } | { is: "mine" } | { is: "other"; name: string };
type Here = "writes" | "keeps" | "asks" | "stops" | "unapproved" | "unread" | "untouched";

export interface Moving {
  from: Scope;
  to: Scope;
  docs: { title: string; file: string; here: Here }[];
  older: string[];
}

const SHOWN = 6;

const HERE_SAID = {
  writes: "agentsMoveWrites",
  keeps: "agentsMoveKeeps",
  asks: "agentsMoveAsks",
  stops: "agentsMoveStops",
  unapproved: "agentsMoveUnapproved",
  unread: "agentsMoveUnread",
} as const;

export const agentsMoving = (
  kind: "doc" | "folder",
  id: string,
  to: string | undefined,
): Promise<Moving | null> => invoke("agents_moving", { kind, id, to });

const place = (scope: Scope): string => {
  if (scope.is === "everyone") return t("agentsMoveEveryone");
  if (scope.is === "mine") return t("agentsMoveMine");
  return fill("agentsMoveOther", scope.name);
};

const elsewhere = ({ from, to }: Moving): string[] => {
  if (to.is === "everyone") return [t("agentsMoveOthersWait")];
  if (to.is === "other") return [fill("agentsMoveToOther", to.name)];
  if (from.is === "everyone") return [t("agentsMoveOthersLeave")];
  if (from.is === "other") return [fill("agentsMoveFromOther", from.name)];
  return [];
};

const hereSaid = (moving: Moving): string[] => {
  const told = moving.docs.filter((one) => one.here !== "untouched");
  const many = moving.docs.length > 1;
  const lines = told.slice(0, SHOWN).map((one) => {
    const said = fill(HERE_SAID[one.here as keyof typeof HERE_SAID], one.file);
    return many ? `«${one.title}»: ${said}` : said;
  });
  if (told.length > SHOWN) lines.push(fill("agentsMoveMore", String(told.length - SHOWN)));
  return lines;
};

export const movingSaid = (moving: Moving): string => {
  const [first] = moving.docs;
  const asked =
    moving.docs.length === 1 && first
      ? fill("agentsMoveOne", first.title, place(moving.to))
      : fill("agentsMoveMany", String(moving.docs.length), place(moving.to));
  const older = moving.older.length ? [fill("agentsMoveOlder", moving.older.join(", "))] : [];
  return [asked, "", ...hereSaid(moving), ...elsewhere(moving), ...older].join("\n");
};

export const movingAllowed = async (
  kind: "doc" | "folder",
  id: string,
  to: string | undefined,
): Promise<boolean> => {
  const moving = await agentsMoving(kind, id, to);
  if (!moving) return true;
  return sure(movingSaid(moving), { yes: t("agentsMoveYes"), no: t("cancel") });
};

export const movedOnceSure = async (
  kind: "doc" | "folder",
  id: string,
  to: string | undefined,
  move: () => Promise<void>,
): Promise<void> => {
  if (await movingAllowed(kind, id, to)) await move();
};
