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
  on,
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
import { DayDetail, ErrorBlock } from "./DayDetail";
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
  interface Remembered {
    /** Everything but the address: subject, configuration, location, weekday. */
    context: string;
    anchor: number;
    offset: number;
    month: MoonMonth | GrahaMonth;
  }

  const months = new Map<string, Remembered>();
  const [loaded, setLoaded] = createSignal(0);

  const MONTH_MEMORY = 32;

  function remember(entry: Remembered) {
    months.set(address(entry.context, entry.anchor, entry.offset), entry);
    while (months.size > MONTH_MEMORY) {
      const oldest = months.keys().next();
      if (oldest.done) break;
      months.delete(oldest.value);
    }
    setLoaded((count) => count + 1);
  }

  /**
   * What a month is an answer to, apart from where it sits.
   *
   * Everything the answer depends on is in here, rather than being cleared out
   * of the map when it changes. The map outlives an open, so a month computed
   * under one ayanamsa or one location must not be handed back under another;
   * naming the inputs makes that impossible instead of remembering to
   * invalidate.
   */
  function context(): string {
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
    ].join("|");
  }

  /** Where a month sits: a context, and a position relative to an anchor. */
  function address(context: string, anchor: number, offset: number): string {
    return `${context}|${anchor}|${offset}`;
  }

  function monthKey(delta: number): string {
    return address(context(), anchor(), offset() + delta);
  }

  /** The month `delta` steps away, or undefined until its first fetch lands. */
  function monthAt(delta: number): MoonMonth | GrahaMonth | undefined {
    loaded();
    return months.get(monthKey(delta))?.month;
  }

  /**
   * Re-addresses every remembered month onto a new anchor.
   *
   * Re-anchoring moves both halves of every address at once, so all three
   * visible months became keys that had never been inserted: nothing rendered
   * and the title emptied until three IPC round trips came back, even though
   * every month involved was already in hand. The months have not changed - only
   * the way they are addressed has - so they are re-addressed rather than
   * refetched. This is I-042 section 2 recurring.
   *
   * Only the current context and the anchor being left are touched; a month
   * remembered under an older configuration is unreachable already and must not
   * be given a current address.
   */
  function readdress(from: number, to: number, shift: number) {
    const here = context();
    const moved: Remembered[] = [];

    for (const [key, entry] of [...months]) {
      if (entry.context !== here || entry.anchor !== from) continue;
      months.delete(key);
      moved.push({ ...entry, anchor: to, offset: entry.offset - shift });
    }
    for (const entry of moved) {
      months.set(address(entry.context, entry.anchor, entry.offset), entry);
    }
    setLoaded((count) => count + 1);
  }

  /**
   * One resource per visible month.
   *
   * The scroller shows three at once, so all three are loaded. The resource is
   * the fetch; what it returns is put in `months` and read from there.
   */
  function monthResource(delta: number) {
    const key = createMemo(() => ({
      subject: subject(),
      context: context(),
      anchor: anchor(),
      offset: offset() + delta,
    }));

    createResource(key, async (current) => {
      if (months.has(address(current.context, current.anchor, current.offset))) {
        return true;
      }
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
        remember({
          context: current.context,
          anchor: current.anchor,
          offset: current.offset,
          month,
        });
        // A month that loads is the answer to the question the error was about.
        if (delta === 0) setError(undefined);
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
   * Counts opens, and nothing else reads it as a number.
   *
   * The scroller is keyed on it so every open builds a fresh one. Its gesture
   * state - the strip's offset, a pending wheel timer, a pending animation frame
   * - lives inside the component and cannot be reached from here, so an open
   * while the panel was already on the calendar left a strip mid-gesture: a
   * resumed settle then committed a month after the offset had been put back to
   * zero. Remounting resets all of it at once, and cannot miss a field the way
   * an imperative reset can.
   */
  const [opened, setOpened] = createSignal(1);

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
    if (!month || Math.abs(offset()) < 6) return;

    const from = anchor();
    const to = month.anchor_unix_ms;
    const shift = offset();
    batch(() => {
      readdress(from, to, shift);
      setAnchor(to);
      setOffset(0);
    });
  });

  /**
   * Which civil day is today depends on the zone, so a location change moves it.
   *
   * Everything computed already follows the new location, because the month
   * address names it; today and the anchor were fixed at construction and
   * refreshed only by an open, so crossing the date line left the today ring on
   * the old zone's date until the panel was closed and reopened.
   */
  createEffect(
    on(
      () => props.boot.location.zone,
      (zone) => {
        const now = todayIn(zone);
        batch(() => {
          setToday(now);
          setAnchor(noonAnchor(now, zone));
          setOffset(0);
        });
      },
      { defer: true },
    ),
  );

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
      const applied = await ipc.updateSettings(next);
      batch(() => {
        props.onSettingsApplied(applied);
        setError(undefined);
      });
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
      // A year with shift, as DESIGN 10.1 specifies. Shift was not filtered out
      // above and was not read either, so it stepped one month in silence.
      PageUp: () => step(event.shiftKey ? -12 : -1),
      PageDown: () => step(event.shiftKey ? 12 : 1),
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
      setOpened((count) => count + 1);
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
          {/* Every view renders the failure it can cause. The signal used to
              reach only the day detail, so a month that failed to load showed
              nothing at all in the calendar, and a settings save that failed
              fired while the user was by definition in settings - which made
              ERROR_TEXT.SETTINGS unreachable text. */}
          <Show when={view() === "calendar"}>
            <Show
              when={error()}
              fallback={
                // Keyed on the open counter: a fresh scroller per open.
                <Show when={opened()} keyed>
                  {(_generation) => (
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
                  )}
                </Show>
              }
            >
              {/* The month in the window is the one that failed - the grid has
                  nothing left to stay live for, which is the case DESIGN 9.3's
                  "the grid stays navigable" does not cover. */}
              {(problem) => (
                <ErrorBlock code={problem().code} message={problem().message} />
              )}
            </Show>
          </Show>

          <Show when={view() === "day"}>
            <DayDetail
              detail={detail()}
              events={selectedEvents()}
              grahaName={grahaInfo()?.name ?? ""}
              context={{ timeZone: timeZone() }}
              isToday={sameDate(selected(), today())}
              error={error()}
            />
          </Show>

          <Show when={view() === "settings"}>
            {/* Above the list rather than instead of it: the controls are what
                the user needs in order to try something else. */}
            <Show when={error()}>
              {(problem) => (
                <ErrorBlock code={problem().code} message={problem().message} />
              )}
            </Show>
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
