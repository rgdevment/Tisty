import { cloudOf } from "../clouds";

export default function CloudMark({ provider, muted }: { provider?: string; muted?: boolean }) {
  const cloud = cloudOf(provider);
  return (
    <span
      className={`grid size-6 flex-none place-items-center rounded-md text-[11.5px] font-semibold text-white ${
        muted ? "bg-soft" : (cloud?.hue ?? "bg-soft")
      }`}
    >
      {cloud?.letter ?? "·"}
    </span>
  );
}
