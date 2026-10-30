import { useEffect, useRef } from "react";
import { type Panel, sided } from "tisty-editor/read";
import { SAY } from "./saying";

// The panels beside a sheet are the sheet's own, drawn by the package as the web draws them; the window only picks which.
export default function SidePanels({
  panels,
  className = "",
}: Readonly<{ panels: readonly Panel[]; className?: string }>) {
  const held = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (held.current) sided(held.current, SAY, panels);
  }, [panels]);
  return <div ref={held} className={`side ${className}`} />;
}
