const HUES: Record<string, [string, string]> = {
  google: ["G", "bg-[#1a73e8]"],
  onedrive: ["O", "bg-[#0364b8]"],
};

export default function CloudMark({ provider, named }: { provider?: string; named?: string }) {
  const [letter, hue] = (provider && HUES[provider]) || [named?.slice(0, 1) ?? "·", "bg-soft"];
  return (
    <span
      className={`grid size-6 flex-none place-items-center rounded-md text-[11.5px] font-semibold text-white ${hue}`}
    >
      {letter}
    </span>
  );
}
