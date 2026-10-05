export default function Section({ label, note }: { label: string; note?: string }) {
  return (
    <div className="mt-5 mb-1.5 flex items-center gap-2.5 text-[11.5px] font-semibold tracking-[0.05em] text-faint uppercase">
      <span>{label}</span>
      <span className="h-px flex-1 bg-hair" />
      {note && <span>{note}</span>}
    </div>
  );
}
