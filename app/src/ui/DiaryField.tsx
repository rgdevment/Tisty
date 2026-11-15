import { useState } from "react";
import { t } from "../locales";
import { saidPlainly } from "../refusal";
import Field from "./Field";

interface Props {
  onWrite: (text: string) => Promise<unknown>;
  onError: (message: string) => void;
}

export default function DiaryField({ onWrite, onError }: Props) {
  const [text, setText] = useState("");
  return (
    <Field
      icon="❧"
      value={text}
      hint={t("diaryWriting")}
      onChange={setText}
      onSubmit={() =>
        onWrite(text)
          .then(() => setText(""))
          .catch((problem) => onError(saidPlainly(problem)))
      }
    />
  );
}
