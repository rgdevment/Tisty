export default function Row({
  glyph,
  say,
  first,
  children,
  onPick,
}: {
  glyph: string;
  say?: string;
  first?: boolean;
  children: React.ReactNode;
  onPick: () => void;
}) {
  return (
    <button
      type="button"
      autoFocus={first}
      onClick={onPick}
      className="flex w-full items-center gap-2.5 rounded-md px-2.5 py-1.5 text-left text-ink hover:bg-hover"
    >
      <span className="w-[15px] text-center">{glyph}</span>
      {children}
      <span className="ml-auto text-[11px] text-faint">{say}</span>
    </button>
  );
}
