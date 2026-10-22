import { Ask } from "./Rows";

interface Props {
  on: boolean;
  label: string;
  why: string;
  onTick: (on: boolean) => void;
}

export default function PdfTick({ on, label, why, onTick }: Props) {
  return (
    <div className="mt-3 flex items-center gap-1.5">
      <label className="flex items-center gap-2.5 text-[13px]">
        <input
          type="checkbox"
          checked={on}
          onChange={(e) => onTick(e.target.checked)}
          className="h-3.5 w-3.5 accent-accent"
        />
        {label}
      </label>
      <Ask said={why} />
    </div>
  );
}
