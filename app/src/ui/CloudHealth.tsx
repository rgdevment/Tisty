import { useCallback, useEffect, useRef, useState } from "react";
import { cloudHealth, type Health } from "../core";
import { fill, t, type Word } from "../locales";

const SAID: Record<Exclude<Health, "reachable">, Word> = {
  blocked: "healthBlocked",
  inspected: "healthInspected",
  slow: "healthSlow",
  troubled: "healthTroubled",
};

export default function CloudHealth({ name, provider }: { name: string; provider: string }) {
  const [health, setHealth] = useState<Health | "checking">("checking");
  const gone = useRef(false);
  const asked = useRef(0);

  // Only the newest look speaks: one for a provider no longer picked may answer late.
  const check = useCallback(() => {
    const mine = ++asked.current;
    const heard = (now: Health | "reachable") =>
      !gone.current && mine === asked.current && setHealth(now);
    setHealth("checking");
    cloudHealth(provider)
      .then((now) => heard(now in SAID ? now : "reachable"))
      .catch(() => heard("reachable"));
  }, [provider]);

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
