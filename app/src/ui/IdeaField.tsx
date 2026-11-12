import { useState } from "react";
import { t } from "../locales";
import { IDEA_AT_MOST } from "../notes";
import { saidPlainly } from "../refusal";
import Field from "./Field";

interface Props {
  onJot: (idea: string) => Promise<unknown>;
  onError: (message: string) => void;
}

export default function IdeaField({ onJot, onError }: Props) {
  const [text, setText] = useState("");
  return (
    <Field
      icon="✎"
      value={text}
      hint={t("jotIdea")}
      most={IDEA_AT_MOST}
      onChange={setText}
      onSubmit={() =>
        onJot(text)
          .then(() => setText(""))
          .catch((problem) => onError(saidPlainly(problem)))
      }
    />
  );
}
