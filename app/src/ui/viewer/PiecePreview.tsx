import type { Held } from "../../pieceWindow";
import Page from "../Page";

type Props = {
  readonly held: Exclude<Held, "widget">;
  readonly source: string;
  readonly label: string;
};

const lined = (source: string) => (source.endsWith("\n") ? source : `${source}\n`);

// The core draws the piece as the document would, alone on its page.
const drawnAs = (held: Props["held"], source: string): string => {
  if (held === "formula") return `\`\`\`math\n${lined(source)}\`\`\`\n`;
  if (held === "inlineFormula") return `$$${source.trim()}$$\n`;
  if (held === "front") return `---\n${lined(source)}---\n`;
  return lined(source);
};

export default function PiecePreview({ held, source, label }: Props) {
  return (
    <Page
      body={drawnAs(held, source)}
      label={label}
      onDoc={() => undefined}
      onShown={(page) => page?.document.documentElement.classList.add("held")}
    />
  );
}
