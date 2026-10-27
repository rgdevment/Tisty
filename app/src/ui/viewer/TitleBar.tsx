import type { ReactNode } from "react";
import { clearOfChrome, onMac } from "../WindowChrome";

export default function TitleBar({
  title,
  under,
  lead,
  children,
}: Readonly<{
  title: string;
  under: string;
  lead?: ReactNode;
  children?: ReactNode;
}>) {
  return (
    <header
      data-tauri-drag-region
      className={`flex h-11 shrink-0 items-center gap-2 border-b border-hair bg-panel ${onMac ? "pl-20" : "pl-3.5"} ${clearOfChrome}`}
    >
      {lead}
      <div data-tauri-drag-region className="mr-1.5 flex min-w-0 flex-1 flex-col leading-tight">
        <b data-tauri-drag-region className="truncate text-[13px] font-semibold">
          {title}
        </b>
        <small data-tauri-drag-region className="mt-0.5 truncate text-[11.5px] text-faint">
          {under}
        </small>
      </div>
      {children}
    </header>
  );
}
