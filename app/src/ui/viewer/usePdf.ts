import { save as intoFile } from "@tauri-apps/plugin-dialog";
import { useEffect, useRef, useState } from "react";
import { keepPdf, type Paper, type Viewed } from "../../core";
import { t } from "../../locales";
import { pdfOfText, signature } from "../printing";

interface Asked {
  ready: boolean;
  body: string;
  leaf: Paper;
  paper: Viewed | null;
  named: string;
  onFail: (problem: unknown) => void;
}

export default function usePdf({ ready, body, leaf, paper, named, onFail }: Asked) {
  const [seeing, setSeeing] = useState<string | null>(null);
  const [signing, setSigning] = useState(true);
  const [making, setMaking] = useState(false);

  const made = async () => {
    const signed = paper && signing ? await signature(paper.file) : undefined;
    return pdfOfText(body, leaf, signed);
  };

  const working = async (work: () => Promise<void>) => {
    if (!ready || making) return;
    setMaking(true);
    try {
      await work();
    } catch (e) {
      onFail(e);
    } finally {
      setMaking(false);
    }
  };

  const preview = () =>
    working(async () => {
      const blob = await made();
      setSeeing((was) => {
        if (was) URL.revokeObjectURL(was);
        return URL.createObjectURL(blob);
      });
    });

  const keep = () =>
    working(async () => {
      const where = await intoFile({
        defaultPath: `${named || t("untitledDoc")}.pdf`,
        filters: [{ name: "PDF", extensions: ["pdf"] }],
      });
      if (!where) return;
      const blob = await made();
      await keepPdf(where, Array.from(new Uint8Array(await blob.arrayBuffer())));
    });

  const shut = () => {
    if (seeing) URL.revokeObjectURL(seeing);
    setSeeing(null);
  };

  const redraw = useRef(preview);
  redraw.current = preview;
  const open = useRef(seeing);
  open.current = seeing;
  useEffect(() => {
    if (open.current && leaf && signing !== undefined) void redraw.current();
  }, [leaf, signing]);

  return { seeing, signing, making, setSigning, preview, keep, shut };
}
