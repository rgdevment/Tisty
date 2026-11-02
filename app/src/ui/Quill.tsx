// The pen the web shows while a link opens, so a window still waiting reads as on its way rather than as empty.
export default function Quill({ said }: { said: string }) {
  return (
    <div role="status" className="flex flex-1 flex-col items-center justify-center gap-5">
      <svg className="quill" viewBox="0 0 160 180" width="160" height="180" aria-hidden="true">
        <rect className="leaf" x="22" y="14" width="116" height="152" rx="10" />
        <path className="line one" d="M42 54h76" />
        <path className="line two" d="M42 78h76" />
        <path className="line three" d="M42 102h76" />
        <path className="line four" d="M42 126h48" />
        <g className="pen">
          <path className="pen-body" d="M-2 -6 L24 -32 L32 -24 L6 2 Z" />
          <path className="pen-nib" d="M-2 -6 L6 2 L-4 4 Z" />
        </g>
      </svg>
      <p className="text-[12.5px] text-soft">{said}</p>
    </div>
  );
}
