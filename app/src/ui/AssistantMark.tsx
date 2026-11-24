import type { Folded } from "../core";
import Glyph from "./Glyph";

// Plain glyphs that tell the assistants apart without borrowing any of their logos.
const MARKS: Record<string, string> = {
  claude: "chat-text",
  codex: "terminal",
  copilot: "braces",
  gemini: "compass",
  cursor: "pointer-arrow",
  dsh: "orbit",
};

const GLOBAL = "global";

// One hue per assistant, off Agents' coral, the diagrams' purple and Ideas' and Personal's; Global keeps its blue.
const HUES: Record<string, { readonly ink: string; readonly pill: string }> = {
  global: { ink: "text-hue-blue", pill: "text-hue-blue bg-hue-blue/10" },
  claude: { ink: "text-hue-orange", pill: "text-hue-orange bg-hue-orange/10" },
  codex: { ink: "text-hue-teal", pill: "text-hue-teal bg-hue-teal/10" },
  copilot: { ink: "text-hue-indigo", pill: "text-hue-indigo bg-hue-indigo/10" },
  gemini: { ink: "text-hue-green", pill: "text-hue-green bg-hue-green/10" },
  cursor: { ink: "text-hue-brown", pill: "text-hue-brown bg-hue-brown/10" },
  dsh: { ink: "text-hue-gray", pill: "text-hue-gray bg-hue-gray/10" },
};

const UNKNOWN = { ink: "text-soft", pill: "text-soft bg-hover" };

export const assistantPill = (id: string): string => (HUES[id] ?? UNKNOWN).pill;
const LEVELS: Record<string, string> = {
  Global: GLOBAL,
  Claude: "claude",
  Codex: "codex",
  Copilot: "copilot",
  Gemini: "gemini",
  Cursor: "cursor",
  "DeepSeek Harness": "dsh",
};

type Drawn = Pick<Folded, "parent" | "name" | "icon"> & Partial<Pick<Folded, "agents">>;

export const levelOf = (folder: Drawn, root: string | null): string | null => {
  const placed = folder.parent === root || folder.agents?.room;
  if (root === null || !placed || folder.icon || folder.agents?.computer) return null;
  return LEVELS[folder.name] ?? null;
};

interface Props {
  readonly id: string;
  readonly className?: string;
}

export function AssistantMark({ id, className = "h-4 w-4" }: Props) {
  const global = id === GLOBAL;
  return (
    <span data-mark={id} className={`flex items-center ${(HUES[id] ?? UNKNOWN).ink}`}>
      <Glyph name={global ? "globe-earth" : (MARKS[id] ?? "terminal")} className={className} />
    </span>
  );
}

interface Folder {
  readonly folder: Drawn;
  readonly root: string | null;
  readonly className?: string;
}

export function FolderMark({ folder, root, className }: Folder) {
  const level = levelOf(folder, root);
  if (level) return <AssistantMark id={level} className={className} />;
  if (folder.agents?.computer)
    return <Glyph name={folder.icon ?? "monitor"} className={className} />;
  return <Glyph name={folder.icon ?? "folder"} className={className} />;
}
