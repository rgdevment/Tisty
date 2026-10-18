const WIDTHS = ["38%", "92%", "84%", "64%", "30%", "88%", "72%"];

// Grey lines in the shape of a page hold its place while it is drawn, so the wait reads as one.
export default function Skeleton({ inset = "36px 40px" }: { inset?: string }) {
  return (
    <div
      aria-hidden="true"
      className="pointer-events-none absolute inset-x-0 top-0 flex flex-col gap-3.5 motion-safe:animate-pulse"
      style={{ padding: inset }}
    >
      {WIDTHS.map((wide, at) => (
        <span
          // biome-ignore lint/suspicious/noArrayIndexKey: the lines are fixed and never move
          key={at}
          className={`block h-3 rounded-full bg-hover ${at % 4 === 0 ? "mt-2" : ""}`}
          style={{ width: wide }}
        />
      ))}
    </div>
  );
}
