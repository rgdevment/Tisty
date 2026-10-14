import { useCallback, useEffect, useRef, useState } from "react";
import { cloudHealth, type Health } from "../core";
import { fill, t, type Word } from "../locales";

const SAID: Record<Exclude<Health, "reachable">, Word> = {
  blocked: "healthBlocked",
  inspected: "healthInspected",
  slow: "healthSlow",
  troubled: "healthTroubled",
};

export default function CloudHealth({ name }: { name: string }) {
  const [health, setHealth] = useState<Health | "checking">("checking");
  const gone = useRef(false);

  const check = useCallback(() => {
    setHealth("checking");
    cloudHealth()
      .then((now) => !gone.current && setHealth(now in SAID ? now : "reachable"))
      .catch(() => !gone.current && setHealth("reachable"));
  }, []);

  useEffect(() => {
    gone.current = false;
    check();
    return () => {
      gone.current = true;
    };
  }, [check]);

  if (health === "reachable") return null;
  if (health === "checking")
    return (
      <p aria-live="polite" className="text-[11.5px] text-faint">
        {fill("healthChecking", name)}
      </p>
    );
  const slow = health === "slow";
  return (
    <div role={slow ? "status" : "alert"} className="flex flex-col gap-1.5">
      <p className={`text-[11.5px] leading-relaxed ${slow ? "text-soft" : "text-urgent"}`}>
        {fill(SAID[health], name)}
      </p>
      <button
        type="button"
        onClick={check}
        className="self-start text-[11.5px] text-accent hover:underline"
      >
        {t("healthAgain")}
      </button>
    </div>
  );
}
