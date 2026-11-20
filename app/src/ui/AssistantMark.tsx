import type { Folded } from "../core";
import Glyph from "./Glyph";

// Plain glyphs that tell the assistants apart without borrowing any of their logos.
const MARKS: Record<string, string> = {
  claude: "chat-text",
  codex: "terminal",
  copilot: "braces",
  gemini: "compass",
  cursor: "pointer-arrow",
};

const GLOBAL = "global";
const LEVELS: Record<string, string> = {
  Global: GLOBAL,
  Claude: "claude",
  Codex: "codex",
  Copilot: "copilot",
  Gemini: "gemini",
  Cursor: "cursor",
};

export const levelOf = (
  folder: Pick<Folded, "parent" | "name" | "icon">,
  root: string | null,
): string | null =>
  root !== null && folder.parent === root && !folder.icon ? (LEVELS[folder.name] ?? null) : null;

interface Props {
  readonly id: string;
  readonly className?: string;
}

export function AssistantMark({ id, className = "h-4 w-4" }: Props) {
  const global = id === GLOBAL;
  return (
    <span data-mark={id} className={`flex items-center ${global ? "text-hue-blue" : "text-soft"}`}>
      <Glyph name={global ? "globe-earth" : (MARKS[id] ?? "terminal")} className={className} />
    </span>
  );
}

interface Folder {
  readonly folder: Pick<Folded, "parent" | "name" | "icon">;
  readonly root: string | null;
  readonly className?: string;
}

export function FolderMark({ folder, root, className }: Folder) {
  const level = levelOf(folder, root);
  if (level) return <AssistantMark id={level} className={className} />;
  return <Glyph name={folder.icon ?? "folder"} className={className} />;
}
