const HUES: Record<string, [string, string]> = {
  google: ["G", "bg-[#1a73e8]"],
  onedrive: ["O", "bg-[#0364b8]"],
};

export default function CloudMark({ provider }: { provider?: string | null }) {
  const [letter, hue] = HUES[provider ?? "google"] ?? ["·", "bg-soft"];
  return (
    <span
      className={`grid size-6 flex-none place-items-center rounded-md text-[11.5px] font-semibold text-white ${hue}`}
    >
      {letter}
    </span>
  );
}
