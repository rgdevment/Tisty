import type { Papers, Snapshot, Task } from "../core";
import type { detailOf } from "../detailing";
import { saidPlainly } from "../refusal";
import type { Chosen } from "../views";
import Detail from "./Detail";
import Door from "./Door";
import Pulse from "./Pulse";
import Star from "./Star";

interface Props {
  aside: boolean;
  beside: boolean;
  dealing: boolean;
  task: Task | undefined;
  detailing: (one: Task) => ReturnType<typeof detailOf>;
  remember: (next: "columns" | "sheet") => void;
  data: Snapshot;
  papers: Papers;
  chosen: Chosen;
  setChosen: (chosen: Chosen) => void;
  setSelected: (id: string | undefined) => void;
  offering: boolean;
  setOffering: (offering: boolean) => void;
  starring: boolean;
  setStarring: (starring: boolean) => void;
  quiet: boolean;
  setError: (text: string | null) => void;
}

export default function Margins({
  aside,
  beside,
  dealing,
  task,
  detailing,
  remember,
  data,
  papers,
  chosen,
  setChosen,
  setSelected,
  offering,
  setOffering,
  starring,
  setStarring,
  quiet,
  setError,
}: Props) {
  return (
    <>
      {beside && task && (
        <Detail
          apart={`${aside ? "right-3 @min-[1536px]:right-[324px]" : "right-3"} ${
            dealing ? "pointer-events-none opacity-10" : ""
          }`}
          key={task.id}
          {...detailing(task)}
          expanded={false}
          onExpand={() => remember("sheet")}
          onCollapse={() => remember("columns")}
        />
      )}
      {aside && (
        <Pulse
          apart={beside ? "hidden @min-[1536px]:flex" : "hidden @min-[884px]:flex"}
          counts={data.counts}
          lists={data.lists}
          ahead={data.ahead ?? []}
          routines={data.routines ?? []}
          papers={papers.docs.filter((one) => !one.pageOf).length}
          onOpen={(id) => setSelected(id)}
          onList={(id) => {
            setSelected(undefined);
            setChosen({ named: "lists", list: id });
          }}
          onQuadrants={() => {
            setSelected(undefined);
            setChosen({ named: "quadrants" });
          }}
        />
      )}

      {offering && quiet && (aside || chosen.named === "docs") && (
        <Door
          apart={
            aside ? "right-3 @min-[884px]:right-[324px]" : "right-3 @min-[1440px]:right-[344px]"
          }
          onSettled={() => setOffering(false)}
          onOpen={() => setChosen({ named: "keeping", tab: "agents" })}
          onError={(problem) => setError(saidPlainly(problem))}
        />
      )}

      {starring && !offering && quiet && (aside || chosen.named === "docs") && (
        <Star
          apart={
            aside ? "right-3 @min-[884px]:right-[324px]" : "right-3 @min-[1440px]:right-[344px]"
          }
          onSettled={() => setStarring(false)}
          onError={(problem) => setError(saidPlainly(problem))}
        />
      )}
    </>
  );
}
