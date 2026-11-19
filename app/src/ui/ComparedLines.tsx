import type { Line } from "../agentsSync";

interface Props {
  readonly lines: readonly Line[];
  readonly gone: string;
  readonly came: string;
}

const MARK = { same: " ", tisty: "−", file: "+" } as const;
const TONE = { same: "text-soft", tisty: "text-hue-red", file: "text-hue-green" } as const;

export default function ComparedLines({ lines, gone, came }: Props) {
  return (
    <div className="mt-3 rounded-[10px] border border-hair">
      <div className="flex gap-4 border-b border-hair px-3 py-1.5 text-[11.5px] text-faint">
        <span className="text-hue-red">− {gone}</span>
        <span className="text-hue-green">+ {came}</span>
      </div>
      <pre className="max-h-64 overflow-auto px-3 py-2 font-mono text-[11.5px] leading-relaxed whitespace-pre-wrap [overflow-wrap:anywhere]">
        {lines.map((line, n) => (
          // biome-ignore lint/suspicious/noArrayIndexKey: a compared line is told once and never moves
          <span key={`${n}-${line.side}`} className={`block ${TONE[line.side]}`}>
            {MARK[line.side]} {line.text}
          </span>
        ))}
      </pre>
    </div>
  );
}
