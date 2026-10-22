import { type FormEvent, useState } from "react";
import type { Connected } from "../core";
import { cloudConnectDav } from "../core";
import { t } from "../locales";
import { saidPlainly } from "../refusal";

interface Props {
  // Reconnecting names the account already kept, so only the password is asked again.
  account?: string;
  busy: boolean;
  onConnected: (account: Connected) => void;
  onTrouble: (said?: string) => void;
  // Told while the server is checked, so nothing around the form starts another sign-in meanwhile.
  onChecking?: (checking: boolean) => void;
}

const field =
  "w-full rounded-[10px] border border-line bg-transparent px-3 py-1.5 text-[13px] text-ink placeholder:text-faint focus:border-accent focus:outline-none disabled:opacity-60";
const label = "flex flex-col gap-1 text-[11.5px] text-soft";
const go =
  "ml-auto rounded-[10px] bg-accent px-4 py-2 text-[13px] font-medium text-white hover:opacity-90 disabled:opacity-60";

export default function DavForm({ account, busy, onConnected, onTrouble, onChecking }: Props) {
  const [address, setAddress] = useState("");
  const [user, setUser] = useState("");
  const [password, setPassword] = useState("");
  const [checking, setChecking] = useState(false);
  const held = busy || checking;

  const check = (now: boolean) => {
    setChecking(now);
    onChecking?.(now);
  };

  const connect = (e: FormEvent) => {
    e.preventDefault();
    if (held) return;
    onTrouble(undefined);
    check(true);
    const signIn = account ? { account, password } : { address, user, password };
    cloudConnectDav(signIn)
      .then((connected) => {
        setPassword("");
        onConnected(connected);
      })
      .catch((e) => onTrouble(saidPlainly(e)))
      .finally(() => check(false));
  };

  return (
    <form onSubmit={connect} className="flex flex-col gap-2.5">
      {account === undefined && (
        <>
          <label className={label}>
            {t("davAddress")}
            <input
              type="url"
              value={address}
              disabled={held}
              required
              placeholder="https://"
              autoComplete="url"
              spellCheck={false}
              onChange={(e) => setAddress(e.target.value)}
              className={field}
            />
          </label>
          <label className={label}>
            {t("davUser")}
            <input
              value={user}
              disabled={held}
              required
              autoComplete="username"
              spellCheck={false}
              onChange={(e) => setUser(e.target.value)}
              className={field}
            />
          </label>
        </>
      )}
      <label className={label}>
        {t("davPassword")}
        <input
          type="password"
          value={password}
          disabled={held}
          required
          autoComplete="current-password"
          onChange={(e) => setPassword(e.target.value)}
          className={field}
        />
        <span className="text-faint">{t("davPasswordWhy")}</span>
      </label>
      {checking && (
        <p role="status" aria-live="polite" className="text-[11.5px] text-faint">
          {t("davChecking")}
        </p>
      )}
      <div className="flex items-center">
        <button type="submit" disabled={held} className={go}>
          {t("davConnect")}
        </button>
      </div>
    </form>
  );
}
