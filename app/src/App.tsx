import { docFile, folderFile } from "./core";
import { DeskProvider, deskOf } from "./desk";
import { hereChoices } from "./docMenus";
import { t } from "./locales";
import { saidPlainly } from "./refusal";
import { WEEK } from "./ui/Ahead";
import Layers from "./ui/Layers";
import Margins from "./ui/Margins";
import Sidebar from "./ui/Sidebar";
import Spine from "./ui/Spine";
import Stage from "./ui/Stage";
import WindowChrome from "./ui/WindowChrome";
import { useWindow } from "./windowing";

export default function App() {
  const held = useWindow();
  const { data, error } = held;
  if (!data) {
    return (
      <div className="grid h-full font-sans" style={{ gridTemplateColumns: "1fr" }}>
        <WindowChrome />
        {error && <p className="mt-16 px-6 text-center text-[11.5px] text-urgent">{error}</p>}
      </div>
    );
  }
  const desk = deskOf(held, data);
  const {
    papers,
    chosen,
    ready,
    here,
    menu,
    hands,
    setHere,
    setChosen,
    setCameFrom,
    setSelected,
    setFound,
    setError,
    setMenu,
    papersChanged,
    hangIt,
    folderMenu,
    docMenu,
    hereMenu,
    papered,
    lane,
    aside,
    beside,
  } = desk;

  return (
    <DeskProvider value={desk}>
      <div className="grid h-full bg-rail font-sans [grid-template-columns:336px_minmax(0,1fr)] min-[1440px]:[grid-template-columns:380px_minmax(0,1fr)]">
        <WindowChrome />

        <Layers parcels={desk.parcels.shown} />

        <Sidebar
          lists={data.lists}
          papers={papers}
          counts={data.counts}
          chosen={chosen}
          waiting={ready ? ready.version || t("updateWaiting") : undefined}
          here={here}
          acting={menu?.on ?? null}
          onHere={(folder) => {
            setHere(folder ?? null);
            setChosen({ named: "docs" });
          }}
          onMove={(folder, parent, before) =>
            folderFile(folder, parent, before)
              .then(papersChanged)
              .catch((e) => setError(saidPlainly(e)))
          }
          onFile={(doc, folder, before) =>
            docFile(doc, folder, before)
              .then(papersChanged)
              .catch((e) => setError(saidPlainly(e)))
          }
          onPage={(doc, pageOf) => hangIt(doc, pageOf)}
          onFolderMenu={folderMenu}
          onDocMenu={docMenu}
          onHereMenu={hereMenu}
          onDocsMenu={(at) =>
            setMenu({ at, label: t("docsActions"), choices: hereChoices(hands, here ?? undefined) })
          }
          onChoose={(next) => {
            setChosen(next);
            setCameFrom(null);
            setSelected(undefined);
            setFound(null);
            setError(null);
          }}
        />

        <div
          className={`@container my-2 mr-2 flex min-w-0 flex-col overflow-hidden rounded-[10px] border border-hair shadow-lift ${
            papered ? "bg-desk" : "bg-bg"
          }`}
        >
          <div className="relative flex min-h-0 w-full min-w-0 flex-1 overflow-hidden">
            <div
              className={`grid min-h-0 min-w-0 flex-1 grid-rows-[minmax(0,1fr)] overflow-hidden motion-safe:transition-[padding] motion-safe:duration-150 ${lane}`}
            >
              <Stage />
            </div>

            <Margins />
          </div>

          {aside && (
            <div className={beside ? "hidden @max-[1536px]:block" : "hidden @max-[884px]:block"}>
              <Spine
                coming={data.ahead ?? []}
                routines={data.routines ?? []}
                days={WEEK}
                onOpen={(id) => setSelected(id)}
              />
            </div>
          )}
        </div>
      </div>
    </DeskProvider>
  );
}
