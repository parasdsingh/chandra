/**
 * The panel.
 *
 * One surface, fixed size, with three views swapping inside a region of
 * constant height: the calendar, a day, and settings. Nothing opens a second
 * window and nothing changes the panel's height, so the popover never resizes
 * under the pointer.
 */

import type { JSX } from "solid-js";
import {
  batch,
  createEffect,
  createMemo,
  createResource,
  createSignal,
  onCleanup,
  onMount,
  Show,
} from "solid-js";

import { listen } from "@tauri-apps/api/event";

import * as ipc from "../ipc";
import type {
  Bootstrap,
  DateKey,
  DayDetail as Detail,
  GrahaKey,
  GrahaMonth,
  MoonMonth,
  Settings,
  Snapshot,
} from "../ipc/types";
import { isAppError } from "../ipc/types";
import { addDays, noonAnchor, sameDate, todayIn } from "../lib/calendar";
import { localeFirstWeekday } from "../lib/format";
import { CalendarScroller } from "./CalendarScroller";
import { DayDetail } from "./DayDetail";
import { Header } from "./Header";
import { SECTION_TITLES, SettingsView, type SettingsSection } from "./SettingsView";

type View = "calendar" | "day" | "settings";

interface Props {
  boot: Bootstrap;
  onSettingsApplied: (next: Bootstrap) => void;
}

export function Panel(props: Props): JSX.Element {
  const timeZone = () => props.boot.location.zone;

  /**
   * Which subject the panel is showing, and which day it calls today.
   *
   * Both are set when the panel opens rather than read live. The page is never
   * reloaded now, so a `new Date()` evaluated during render would leave a panel
   * left open overnight ringing yesterday.
   */
  const [subject, setSubject] = createSignal<GrahaKey>(props.boot.subject);
  const [today, setToday] = createSignal<DateKey>(todayIn(props.boot.location.zone));

  // Navigation is by anchor instant plus offset, because a lunar month has no
  // year-and-number to step through.
  const [anchor, setAnchor] = createSignal(
    noonAnchor(todayIn(props.boot.location.zone), props.boot.location.zone),
  );
  const [offset, setOffset] = createSignal(0);
  const [view, setView] = createSignal<View>("calendar");
  const [section, setSection] = createSignal<SettingsSection>("root");
  const [selected, setSelected] = createSignal<DateKey | null>(null);
  const [error, setError] = createSignal<{ code: string; message: string }>();

  const firstWeekday = localeFirstWeekday();

  /**
   * Months already fetched, held by identity.
   *
   * A Solid resource drops to `undefined` while it refetches, so reading the
   * grid straight from three resources emptied all three the instant a scroll
   * committed: the strip went blank and the title cleared until the round trips
   * came back, even though every month involved was already known. Holding them
   * by identity means committing promotes a month that is already in hand and
   * the frame after a commit is identical to the frame before it.
   *
   * Bounded, because navigation re-anchors and old keys are then unreachable.
   */
  const months = new Map<string, MoonMonth | GrahaMonth>();
  const [loaded, setLoaded] = createSignal(0);

  const MONTH_MEMORY = 32;

  function remember(key: string, month: MoonMonth | GrahaMonth) {
    months.set(key, month);
    while (months.size > MONTH_MEMORY) {
      const oldest = months.keys().next();
      if (oldest.done) break;
      months.delete(oldest.value);
    }
    setLoaded((count) => count + 1);
  }

  /**
   * Identity of a month.
   *
   * Everything the answer depends on is in the key, rather than being cleared
   * out of the map when it changes. The map now outlives an open, so a month
   * cached under one ayanamsa or one location must not be handed back under
   * another; naming the inputs makes that impossible instead of remembering to
   * invalidate.
   */
  function monthKey(delta: number): string {
    const location = props.boot.location;
    return [
      subject(),
      props.boot.settings.calendar.month_system,
      props.boot.settings.sidereal.ayanamsa,
      props.boot.settings.sidereal.node_type,
      location.zone,
      location.latitude,
      location.longitude,
      location.elevation,
      firstWeekday,
      anchor(),
      offset() + delta,
    ].join("|");
  }

  /** The month `delta` steps away, or undefined until its first fetch lands. */
  function monthAt(delta: number): MoonMonth | GrahaMonth | undefined {
    loaded();
    return months.get(monthKey(delta));
  }

  /**
   * One resource per visible month.
   *
   * The scroller shows three at once, so all three are loaded. The resource is
   * the fetch; what it returns is put in `months` and read from there.
   */
  function monthResource(delta: number) {
    const key = createMemo(() => ({
      id: monthKey(delta),
      subject: subject(),
      anchor: anchor(),
      offset: offset() + delta,
    }));

    createResource(key, async (current) => {
      if (months.has(current.id)) return true;
      try {
        const month =
          current.subject === "chandra"
            ? ((await ipc.moonMonth(
                current.anchor,
                current.offset,
                firstWeekday,
              )) as MoonMonth)
            : ((await ipc.grahaMonth(
                current.subject,
                current.anchor,
                current.offset,
                firstWeekday,
              )) as GrahaMonth);
        remember(current.id, month);
      } catch (thrown) {
        if (delta === 0) setError(toError(thrown));
      }
      return true;
    });
  }

  monthResource(-1);
  monthResource(0);
  monthResource(1);

  const monthData = () => monthAt(0);

  /**
   * Which month the strip is showing while it moves, before it has committed.
   *
   * Kept here rather than in the scroller so the header title changes at the
   * moment the new month takes over the window.
   */
  const [visibleDelta, setVisibleDelta] = createSignal(0);

  /**
   * Keeps the offset small.
   *
   * Every fetch resolves the month by stepping `offset` months from the anchor,
   * and for a lunar month each step is a syzygy search. Left to grow, scrolling
   * a few years would make every fetch walk dozens of syzygies. Re-anchoring on
   * the month in view resets the walk to nothing and resolves to the same
   * months, so the cache still hits and nothing on screen changes.
   */
  createEffect(() => {
    const month = monthAt(0);
    if (month && Math.abs(offset()) >= 6) {
      batch(() => {
        setAnchor(month.anchor_unix_ms);
        setOffset(0);
      });
    }
  });

  const [snapshot, { refetch: refetchSnapshot }] = createResource(async () => {
    try {
      return (await ipc.snapshot(Date.now())) as Snapshot;
    } catch {
      // The header glyph is decoration; failing to draw it must not cost the
      // grid, which is what the panel was opened for.
      return undefined;
    }
  });

  const detailKey = createMemo(() => {
    const date = selected();
    return date && view() === "day" ? { subject: subject(), date } : null;
  });

  const [detail] = createResource(detailKey, async (key) => {
    if (!key) return undefined;
    setError(undefined);
    try {
      return (await ipc.dayDetail(
        key.subject,
        key.date.year,
        key.date.month,
        key.date.day,
      )) as Detail;
    } catch (thrown) {
      setError(toError(thrown));
      return undefined;
    }
  });

  const grahaInfo = createMemo(() =>
    props.boot.grahas.find((graha) => graha.key === subject()),
  );

  /**
   * Events for the selected day, read from the month that owns it.
   *
   * A month's events are built from its own days, so the leading and trailing
   * cells have none in the month being displayed - they belong to the
   * neighbours. Filtering the centre month alone showed an ingress on the 3rd in
   * September and nothing on the same 3rd drawn at the foot of August.
   */
  const selectedEvents = createMemo(() => {
    const date = selected();
    if (!date || subject() === "chandra") return [];

    for (const delta of [0, -1, 1]) {
      const data = monthAt(delta) as GrahaMonth | undefined;
      if (!data) continue;
      const owns = data.days.some(
        (day) => day.in_month && sameDate(day.date, date),
      );
      if (!owns) continue;
      return data.events.filter((event) => sameDate(event.date, date));
    }
    return [];
  });

  function step(delta: number) {
    if (delta === 0) return;
    setOffset((current) => current + delta);
  }

  /** Moves the selection, following it into the neighbouring month. */
  function moveSelection(deltaDays: number) {
    const from = selected() ?? today();
    const next = addDays(from, deltaDays);
    setSelected(next);

    const days = monthData()?.days;
    const inside =
      days?.some(
        (day) =>
          day.date.year === next.year &&
          day.date.month === next.month &&
          day.date.day === next.day,
      ) ?? false;

    if (!inside) {
      batch(() => {
        setAnchor(noonAnchor(next, timeZone()));
        setOffset(0);
      });
    }
  }

  function openDay(date: DateKey) {
    batch(() => {
      setSelected(date);
      setView("day");
    });
  }

  function back() {
    batch(() => {
      if (view() === "settings" && section() !== "root") {
        setSection("root");
        return;
      }
      setView("calendar");
      setSection("root");
    });
  }

  function jumpToToday() {
    const now = today();
    batch(() => {
      setAnchor(noonAnchor(now, timeZone()));
      setOffset(0);
      setSelected(now);
      setView("calendar");
    });
  }

  async function applySettings(next: Settings) {
    try {
      props.onSettingsApplied(await ipc.updateSettings(next));
    } catch (thrown) {
      setError(toError(thrown));
    }
  }

  // ------------------------------------------------------------------ keyboard

  function onKeyDown(event: KeyboardEvent) {
    if (event.metaKey && event.key.toLowerCase() === "w") {
      event.preventDefault();
      void ipc.closePanel();
      return;
    }
    if (event.metaKey && event.key === ",") {
      event.preventDefault();
      batch(() => {
        setView("settings");
        setSection("root");
      });
      return;
    }
    if (event.metaKey || event.ctrlKey || event.altKey) return;

    if (event.key === "Escape") {
      event.preventDefault();
      if (view() !== "calendar") back();
      else if (selected()) setSelected(null);
      else void ipc.closePanel();
      return;
    }

    if (view() !== "calendar") return;

    const handlers: Record<string, () => void> = {
      ArrowLeft: () => moveSelection(-1),
      ArrowRight: () => moveSelection(1),
      ArrowUp: () => moveSelection(-7),
      ArrowDown: () => moveSelection(7),
      PageUp: () => step(-1),
      PageDown: () => step(1),
      Home: () => selectEdge("first"),
      End: () => selectEdge("last"),
      Enter: () => openDay(selected() ?? today()),
      " ": () => openDay(selected() ?? today()),
      t: jumpToToday,
      T: jumpToToday,
    };

    const handler = handlers[event.key];
    if (!handler) return;
    // Unbound keys are silently ignored: no beep, no shake.
    event.preventDefault();
    handler();
  }

  /**
   * First or last day of the displayed month, as DESIGN specifies.
   *
   * Not the first or last cell of the grid: those belong to the neighbouring
   * months, so Home in August selected 27 July and opened a day detail under an
   * August header.
   */
  function selectEdge(edge: "first" | "last") {
    const days = monthData()?.days.filter((day) => day.in_month);
    if (!days || days.length === 0) return;
    setSelected((edge === "first" ? days[0] : days[days.length - 1])!.date);
  }

  /**
   * Everything an open used to get for free by reloading the page.
   *
   * The panel opens on the calendar, at today, with nothing selected. Written
   * out because the page now survives between opens: what a fresh document gave
   * implicitly has to be done on purpose.
   *
   * Idempotent, because it is called twice for every open - once from the event
   * the backend sends as it shows the window, once when the window takes focus.
   */
  function open(next: GrahaKey) {
    const now = todayIn(props.boot.location.zone);
    batch(() => {
      setSubject(next);
      setToday(now);
      setAnchor(noonAnchor(now, timeZone()));
      setOffset(0);
      setVisibleDelta(0);
      setSelected(null);
      setView("calendar");
      setSection("root");
      setError(undefined);
    });
    // The header's phase glyph is a live reading, not a property of the day.
    void refetchSnapshot();
    root?.focus();
  }

  let root: HTMLDivElement | undefined;
  onMount(() => {
    root?.focus();
    const listener = (event: KeyboardEvent) => onKeyDown(event);
    window.addEventListener("keydown", listener);
    onCleanup(() => window.removeEventListener("keydown", listener));

    const opened = listen<string>("chandra://open", (event) =>
      open(event.payload as GrahaKey),
    );
    onCleanup(() => void opened.then((unlisten) => unlisten()));
  });

  const headerTitle = () => {
    if (view() === "settings") return SECTION_TITLES[section()];
    if (view() === "day") return "";
    return monthAt(visibleDelta())?.label ?? "";
  };

  return (
    <div class="panel-frame">
      <div
        class="panel"
        classList={{ "is-opaque": !props.boot.panel_material }}
        ref={root}
        tabindex="-1"
        role="dialog"
        aria-label="Chandra"
      >
        <Header
          subject={subject()}
          subjectName={grahaInfo()?.name ?? "Chandra"}
          info={grahaInfo()}
          snapshot={snapshot()}
          southern={props.boot.location.latitude < 0}
          title={headerTitle()}
          selected={selected()}
          view={view()}
          onBack={back}
          onSettings={() =>
            batch(() => {
              setView("settings");
              setSection("root");
            })
          }
        />

        <div class="region">
          <Show when={view() === "calendar"}>
            <CalendarScroller
              firstWeekday={firstWeekday}
              previous={monthAt(-1)}
              current={monthAt(0)}
              next={monthAt(1)}
              kind={subject() === "chandra" ? "moon" : "graha"}
              info={grahaInfo()}
              selected={selected()}
              today={today()}
              southern={props.boot.location.latitude < 0}
              onSelect={openDay}
              onCommit={(delta) => setOffset((current) => current + delta)}
              onVisibleChange={setVisibleDelta}
            />
          </Show>

          <Show when={view() === "day"}>
            <DayDetail
              detail={detail()}
              events={selectedEvents()}
              grahaName={grahaInfo()?.name ?? ""}
              context={{
                timeZone: timeZone(),
                timeFormat: props.boot.settings.time_format,
              }}
              isToday={sameDate(selected(), today())}
              error={error()}
            />
          </Show>

          <Show when={view() === "settings"}>
            <SettingsView
              boot={props.boot}
              section={section()}
              onOpen={setSection}
              apply={(next) => void applySettings(next)}
            />
          </Show>
        </div>

      </div>
    </div>
  );
}

function toError(thrown: unknown): { code: string; message: string } {
  if (isAppError(thrown)) return { code: thrown.code, message: thrown.message };
  return { code: "ENGINE", message: String(thrown) };
}
