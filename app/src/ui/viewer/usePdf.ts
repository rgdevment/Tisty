import { save as intoFile } from "@tauri-apps/plugin-dialog";
import { useEffect, useRef, useState } from "react";
import { keepFile, type Paper, type Viewed } from "../../core";
import { landscapeIn } from "../../front";
import { t } from "../../locales";
import { usePreviewing } from "../previewing";
import { pdfOf, stampOf } from "../printing";

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
  const [carding, setCarding] = useState(true);
  const [crediting, setCrediting] = useState(true);
  const [making, setMaking] = useState(false);
  usePreviewing(Boolean(seeing));

  const made = async () => {
    const stamp = paper
      ? await stampOf(paper.file, body, { carded: carding, credited: crediting })
      : { credited: crediting };
    return pdfOf([body], leaf, stamp, landscapeIn(body));
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
      await keepFile(where, Array.from(new Uint8Array(await blob.arrayBuffer())));
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
    if (open.current && leaf && carding !== undefined && crediting !== undefined)
      void redraw.current();
  }, [leaf, carding, crediting]);

  return {
    seeing,
    carding,
    crediting,
    making,
    setCarding,
    setCrediting,
    preview,
    keep,
    shut,
  };
}
