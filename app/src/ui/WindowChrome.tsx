import { getCurrentWindow } from "@tauri-apps/api/window";
import { t } from "../locales";

export const onMac = navigator.userAgent.includes("Macintosh");

export default function WindowChrome() {
  const win = getCurrentWindow();

  const buttons: { key: "minimise" | "maximise" | "close"; glyph: string; act: () => void; danger?: boolean }[] = [
    { key: "minimise", glyph: "─", act: () => void win.minimize() },
    { key: "maximise", glyph: "□", act: () => void win.toggleMaximize() },
    { key: "close", glyph: "✕", act: () => void win.close(), danger: true },
  ];

  return (
    <div
      className={`fixed top-0 z-50 flex h-9 items-center gap-0.5 px-2 ${onMac ? "left-0" : "right-0"}`}
    >
      {buttons.map((button) => (
        <button
          key={button.key}
          onClick={button.act}
          title={t(button.key)}
          className={`grid h-7 w-9 place-items-center rounded-md text-[11px] text-soft ${
            button.danger ? "hover:bg-urgent hover:text-white" : "hover:bg-hover"
          }`}
        >
          {button.glyph}
        </button>
      ))}
    </div>
  );
}
