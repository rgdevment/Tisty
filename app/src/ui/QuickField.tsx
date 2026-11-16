import { useState } from "react";
import { saidPlainly } from "../refusal";
import Field from "./Field";

interface Props {
  icon: string;
  hint: string;
  most?: number;
  onKeep: (text: string) => Promise<unknown>;
  onError: (message: string) => void;
}

export default function QuickField({ icon, hint, most, onKeep, onError }: Props) {
  const [text, setText] = useState("");
  return (
    <Field
      icon={icon}
      value={text}
      hint={hint}
      most={most}
      onChange={setText}
      onSubmit={() =>
        onKeep(text)
          .then(() => setText(""))
          .catch((problem) => onError(saidPlainly(problem)))
      }
    />
  );
}
