import { capture, fold } from "../core";
import { useDesk } from "../desk";
import { todayLong } from "../format";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { accepts, asView, headerCount, invite, nothing, title } from "../views";
import { ArchiveBar, SliceBar } from "./Bars";
import CaptureField from "./CaptureField";
import Search from "./Search";
import Shelf from "./Shelf";
import Sightings from "./Sightings";
import Tagged from "./Tagged";
import Tags from "./Tags";
import TaskList from "./TaskList";

export default function Board() {
  const {
    act,
    asking,
    byList,
    cameFrom,
    captured,
    chosen,
    data,
    found,
    further,
    load,
    marking,
    openDoc,
    reveal,
    say,
    seen,
    selected,
    setCameFrom,
    setCaptured,
    setChosen,
    setError,
    setFound,
    setSelected,
    shown,
    strip,
    taggedDocs,
    wholes,
  } = useDesk();
  return (
    <div className="flex min-w-0">
      <div className="flex min-w-0 flex-1 flex-col overflow-hidden">
        <TaskList
          tasks={shown}
          lists={data.every ?? data.lists}
          wholes={wholes}
          title={title(chosen, data.lists)}
          when={chosen.named === "tasks" ? todayLong() : undefined}
          count={headerCount(chosen, found, data.counts, data.total)}
          onBack={
            chosen.list
              ? () => {
                  setChosen({ named: "lists" });
                  setSelected(undefined);
                }
              : chosen.named === "tags"
                ? () => {
                    setChosen(cameFrom ?? { named: "tasks" });
                    setCameFrom(null);
                    setSelected(undefined);
                  }
                : undefined
          }
          empty={
            found?.papers.length && !shown.length
              ? t("onlyPapers")
              : nothing(seen, found !== null, data.counts.tracesHidden ?? 0)
          }
          onReach={!found && data.total > data.tasks.length ? further : undefined}
          note={
            found && found.total > found.tasks.length
              ? fill("someOfMany", `${found.tasks.length}/${found.total}`)
              : undefined
          }
          selected={selected}
          fresh={captured?.id}
          reveal={reveal}
          bands={
            found !== null || chosen.list || chosen.named === "tags" || chosen.tags?.length
              ? undefined
              : chosen.named === "archive"
                ? "month"
                : "day"
          }
          axis={found === null && chosen.named === "archive" ? chosen.axis : undefined}
          dense={
            found === null &&
            chosen.named === "archive" &&
            !chosen.folded &&
            chosen.layer === "trace"
          }
          onSelect={setSelected}
          onComplete={
            chosen.named === "archive"
              ? undefined
              : (id) => {
                  const one = shown.find((task) => task.id === id);
                  marking(id, one?.title ?? "");
                  if (id === selected) setSelected(undefined);
                }
          }
          onFold={chosen.named === "archive" ? (id, away) => act(fold(id, away)) : undefined}
          closing={asking?.id}
          ask={(id) => (asking?.id === id ? strip : null)}
          below={
            found?.papers.length ? (
              <Sightings papers={found.papers} onOpen={openDoc} />
            ) : chosen.tags?.length ? (
              <Tagged docs={taggedDocs} onOpen={openDoc} />
            ) : undefined
          }
          instead={
            chosen.named === "archive" &&
            !chosen.folded &&
            chosen.layer === "routine" &&
            found === null ? (
              <Shelf
                lists={data.lists}
                onOpen={setSelected}
                onError={(e) => setError(saidPlainly(e))}
              />
            ) : undefined
          }
          above={
            chosen.named === "tasks" ? (
              <SliceBar
                chosen={chosen}
                counts={data.counts}
                lists={byList ? [] : data.lists}
                setChosen={setChosen}
                setSelected={setSelected}
              />
            ) : chosen.named === "archive" ? (
              <ArchiveBar
                chosen={chosen}
                counts={data.counts}
                found={found}
                setChosen={setChosen}
                setSelected={setSelected}
                setFound={setFound}
                fail={(e) => setError(saidPlainly(e))}
              />
            ) : chosen.named === "tags" || chosen.tags?.length ? (
              <Tags
                tags={data.tags}
                chosen={chosen.tags ?? []}
                onToggle={(tag) => {
                  const now = chosen.tags ?? [];
                  const next = now.includes(tag) ? now.filter((t) => t !== tag) : [...now, tag];
                  setChosen({ named: "tags", tags: next });
                  setSelected(undefined);
                }}
              />
            ) : undefined
          }
        >
          {chosen.named === "search" ? (
            <Search key="search" onFound={setFound} onError={setError} />
          ) : chosen.named === "archive" ? (
            <Search key="archive" fixed="archived" onFound={setFound} onError={setError} />
          ) : accepts(chosen) ? (
            <CaptureField
              invite={invite(chosen, data.lists)}
              lists={data.lists}
              tags={data.tags}
              onCapture={(written, edits) => {
                setError(null);
                return capture(written, asView(seen), edits).then((task) => {
                  say(fill("saidFiled", task.title));
                  setCaptured(task);
                  load();
                  return task;
                });
              }}
              onError={setError}
            />
          ) : null}
        </TaskList>
      </div>
    </div>
  );
}
