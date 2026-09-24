import { Component, type ReactNode } from "react";
import { broke } from "./broke";
import { t } from "./locales";

interface Props {
  children: ReactNode;
}

interface Held {
  fell: boolean;
}

export default class Standing extends Component<Props, Held> {
  state: Held = { fell: false };

  static getDerivedStateFromError(): Held {
    return { fell: true };
  }

  componentDidCatch(why: Error) {
    broke(why.name || "Error", why.message, why.stack);
  }

  render() {
    if (!this.state.fell) return this.props.children;
    return (
      <div className="flex h-full w-full items-center justify-center bg-desk p-8">
        <div className="max-w-[420px] text-center">
          <p className="text-[13px] font-semibold text-ink">{t("windowFell")}</p>
          <p className="mt-2 text-[12.5px] leading-relaxed text-soft">{t("windowFellWhy")}</p>
          <button
            type="button"
            onClick={() => window.location.reload()}
            className="mt-4 rounded-[10px] border border-line px-3 py-1.5 text-[12.5px] text-soft hover:bg-hover hover:text-ink"
          >
            {t("windowFellAgain")}
          </button>
        </div>
      </div>
    );
  }
}
