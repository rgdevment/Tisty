import { Component, type ReactNode } from "react";
import { caught } from "../broke";
import { t } from "../locales";
import { plainly } from "../refusal";

export const Trouble = ({
  said,
  why,
  onAgain,
}: {
  readonly said: string;
  readonly why: string;
  readonly onAgain: () => void;
}) => (
  <div role="alert" className="flex items-center gap-2.5 px-4 py-2 text-[11.5px]">
    <span className="min-w-0 flex-1">
      <span className="block text-soft">{said}</span>
      <span className="block break-words text-faint">{why}</span>
    </span>
    <button type="button" onClick={onAgain} className="flex-none text-accent hover:underline">
      {t("tryAgain")}
    </button>
  </div>
);

interface Props {
  readonly label: string;
  readonly body: string;
  readonly children: ReactNode;
}

interface Held {
  readonly fell: string | null;
  readonly label: string;
  readonly body: string;
}

export default class Stumbled extends Component<Props, Held> {
  state: Held = { fell: null, label: this.props.label, body: this.props.body };

  static getDerivedStateFromError(why: unknown): Partial<Held> {
    return { fell: plainly(why) };
  }

  static getDerivedStateFromProps({ label, body }: Props, held: Held): Partial<Held> | null {
    return label === held.label && body === held.body ? null : { fell: null, label, body };
  }

  componentDidCatch(why: unknown) {
    caught(why);
  }

  render() {
    const { fell } = this.state;
    if (fell === null) return this.props.children;
    return (
      <Trouble said={t("pageFell")} why={fell} onAgain={() => this.setState({ fell: null })} />
    );
  }
}
