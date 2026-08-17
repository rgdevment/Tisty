import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef, useState } from "react";
import { t } from "../locales";

export const onMac = navigator.userAgent.includes("Macintosh");

const ink = "rgb(0 0 0 / 0.58)";

const Crossed = () => (
  <svg viewBox="0 0 10 10" className="h-2.5 w-2.5" aria-hidden="true">
    <path
      d="M3.1 3.1l3.8 3.8M6.9 3.1L3.1 6.9"
      stroke={ink}
      strokeWidth="1.15"
      strokeLinecap="round"
    />
  </svg>
);

const Barred = () => (
  <svg viewBox="0 0 10 10" className="h-2.5 w-2.5" aria-hidden="true">
    <path d="M2.4 5h5.2" stroke={ink} strokeWidth="1.15" strokeLinecap="round" />
  </svg>
);

const Grown = () => (
  <svg viewBox="0 0 10 10" className="h-2.5 w-2.5" aria-hidden="true">
    <path d="M2.7 6.3V3.2h3.1z" fill={ink} />
    <path d="M7.3 3.7v3.1H4.2z" fill={ink} />
  </svg>
);

export default function WindowChrome() {
  const held = useRef<ReturnType<typeof getCurrentWindow>>(null);
  held.current ??= getCurrentWindow();
  const win = held.current;
  const [awake, setAwake] = useState(true);

  useEffect(() => {
    if (!onMac) return;
    let drop: (() => void) | undefined;
    let gone = false;
    void Promise.resolve(
      win.onFocusChanged?.(({ payload }: { payload: boolean }) => setAwake(payload)),
    )
      .then((off) => {
        if (gone) return (off as (() => void) | undefined)?.();
        drop = off as (() => void) | undefined;
      })
      .catch(() => {});
    return () => {
      gone = true;
      try {
        void Promise.resolve(drop?.()).catch(() => {});
      } catch {
        void 0;
      }
    };
  }, [win]);

  const acts = {
    minimise: () => void win.minimize(),
    maximise: () => void win.toggleMaximize(),
    close: () => void win.close(),
  } as const;

  if (onMac) {
    const lights = [
      { key: "close", lit: "bg-[#ff5f57]", woken: "group-hover:bg-[#ff5f57]", mark: <Crossed /> },
      { key: "minimise", lit: "bg-[#febc2e]", woken: "group-hover:bg-[#febc2e]", mark: <Barred /> },
      { key: "maximise", lit: "bg-[#28c840]", woken: "group-hover:bg-[#28c840]", mark: <Grown /> },
    ] as const;

    return (
      <div
        data-chrome
        className="group fixed top-0 left-0 z-50 flex h-9 items-center gap-2 pl-[13px]"
      >
        {lights.map((one) => (
          <button
            type="button"
            key={one.key}
            onClick={acts[one.key]}
            aria-label={t(one.key)}
            title={t(one.key)}
            className={`grid h-3 w-3 place-items-center rounded-full shadow-[inset_0_0_0_0.5px_rgb(0_0_0/0.14)] transition-colors ${
              awake ? one.lit : `bg-line ${one.woken}`
            }`}
          >
            <span className="opacity-0 transition-opacity group-hover:opacity-100">{one.mark}</span>
          </button>
        ))}
      </div>
    );
  }

  const buttons: { key: keyof typeof acts; glyph: string; danger?: boolean }[] = [
    { key: "minimise", glyph: "─" },
    { key: "maximise", glyph: "□" },
    { key: "close", glyph: "✕", danger: true },
  ];

  return (
    <div data-chrome className="fixed top-0 right-0 z-50 flex h-9 items-center gap-0.5 px-2">
      {buttons.map((button) => (
        <button
          type="button"
          key={button.key}
          onClick={acts[button.key]}
          aria-label={t(button.key)}
          title={t(button.key)}
          className={`grid h-7 w-9 place-items-center rounded-md text-[11px] text-soft ${
            button.danger ? "hover:bg-urgent hover:text-bg" : "hover:bg-hover"
          }`}
        >
          <span aria-hidden="true">{button.glyph}</span>
        </button>
      ))}
    </div>
  );
}
