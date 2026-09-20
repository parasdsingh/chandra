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
  VargaKey,
} from "../ipc/types";
import { isAppError } from "../ipc/types";
import { addDays, noonAnchor, sameDate, todayIn } from "../lib/calendar";
import { localeFirstWeekday } from "../lib/format";
import { CalendarScroller } from "./CalendarScroller";
import { MonthJump } from "./MonthJump";
import { LocationGate, locationIsSet } from "./LocationGate";
import { ChartView } from "./ChartView";
import { DayDetail, ErrorBlock } from "./DayDetail";
import { Header } from "./Header";
import {
  SECTION_TITLES,
  SettingsView,
  type SettingsSection,
} from "./SettingsView";

type View = "calendar" | "day" | "settings" | "chart";

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
  const [subject, setSubject] = createSignal<GrahaKey>(
    // A chart key is not a graha. The chart borrows the moon's calendar for the
    // views behind it, which is the one subject always present.
    isChartSubject(props.boot.subject) ? "chandra" : props.boot.subject,
  );

  // Which division the chart shows. It comes from the item that was clicked and
  // not from settings: several charts sit in the menu bar at once, so a setting
  // would give every one of them the same answer.
  const [varga, setVarga] = createSignal<VargaKey>(
    vargaFromKey(props.boot.subject) ?? "d1",
  );
  const [today, setToday] = createSignal<DateKey>(
    todayIn(props.boot.location.zone),
  );

  // Navigation is by anchor instant plus offset, because a lunar month has no
  // year-and-number to step through.
  const [anchor, setAnchor] = createSignal(
    noonAnchor(todayIn(props.boot.location.zone), props.boot.location.zone),
  );
  const [offset, setOffset] = createSignal(0);
  const [view, setView] = createSignal<View>(
    // Each division has its own status item, so the panel can be opened straight
    // onto one. `subject` is a graha key or a `chart:` key; only the second is a
    // view.
    isChartSubject(props.boot.subject) ? "chart" : "calendar",
  );

  /**
   * The chart, refetched on a timer while it is the view.
   *
   * The lagna moves about a degree every four minutes, so a chart left open is
   * wrong within the hour. A minute is finer than the arcminute it prints and
   * coarser than anything a reader would notice moving.
   */
  // A system asking for reduced motion is asked once, and asked again if it
  // changes: somebody who turns it on while the panel is open has asked now.
  const stillness = window.matchMedia("(prefers-reduced-motion: reduce)");
  const [stillPreferred, setStillPreferred] = createSignal(stillness.matches);
  onMount(() => {
    const listen = (event: MediaQueryListEvent) => setStillPreferred(event.matches);
    stillness.addEventListener("change", listen);
    onCleanup(() => stillness.removeEventListener("change", listen));
  });

  const animating = () => props.boot.settings.chart.animate && !stillPreferred();

  /**
   * Whether the panel is on screen.
   *
   * The webview is never reloaded, so a hidden panel is a running page with
   * nobody looking at it. `view()` stays `"chart"` after the panel is
   * dismissed, so the chart's timer kept refetching and relaying out the whole
   * chart every second for the life of the process - the panel's largest
   * standing cost, and invisible. `visibilitychange` does not fire for an
   * AppKit window being ordered out, so the back end says so instead.
   */
  const [shown, setShown] = createSignal(true);

  const [chartAt, setChartAt] = createSignal(Date.now());
  createEffect(() => {
    if (view() !== "chart" || !shown()) return;
    // One second while it moves, a minute while it does not. A chart costs 47
    // microseconds, so a second is 0.005% of a core - and the slowest division
    // takes two hours to cross a compartment, which is 7,200 steps at this rate.
    // Nothing is gained by asking the ephemeris faster than the eye resolves.
    //
    // `Chakra` does not need telling. Its slide is measured from the gap
    // between the last two charts it was handed and clamped to SETTLE_LEAST and
    // SETTLE_MOST, so it follows whatever this interval turns out to be rather
    // than being held equal to it. That is what the round trip requires: the
    // interval fires on time, the IPC does not, so the arrivals are about a
    // second apart but not evenly so. This comment used to ask a maintainer to
    // keep two numbers in step, and the other number had already been deleted.
    const every = animating() ? 1_000 : 60_000;
    const timer = window.setInterval(() => setChartAt(Date.now()), every);
    onCleanup(() => window.clearInterval(timer));
  });

  const [chart] = createResource(
    () =>
      view() === "chart" && locationIsSet(props.boot)
        ? ([chartAt(), varga()] as const)
        : undefined,
    ([at, division]) => ipc.chakra(at, division),
  );
  const [section, setSection] = createSignal<SettingsSection>("root");
  /** Where settings was opened from, so closing it goes back there rather than
   *  always to the calendar - which dropped anyone who opened settings from the
   *  chart onto a view they had not asked for. */
  const [settingsFrom, setSettingsFrom] = createSignal<View>("calendar");
  const [selected, setSelected] = createSignal<DateKey | null>(null);
  // Not a `view`: the header stays, because the header's title is the control
  // that opened this and has to keep saying so.
  const [jumping, setJumping] = createSignal(false);
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
    const key = createMemo(() =>
      // Nothing is fetched while the gate is up. A month computed from the
      // timezone centroid would be wrong in exactly the way the gate exists to
      // prevent - and it would be *cached* wrong, so the first month seen after
      // a location was chosen would still be the guess.
      locationIsSet(props.boot)
        ? {
            subject: subject(),
            context: context(),
            anchor: anchor(),
            offset: offset() + delta,
          }
        : undefined,
    );

    createResource(key, async (current) => {
      if (
        months.has(address(current.context, current.anchor, current.offset))
      ) {
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
        // Back to the section this one hangs from, not always to the top. The
        // advanced panes are two levels down, and returning them to the root
        // skipped the list they were opened from.
        setSection(SETTINGS_PARENT[section()] ?? "root");
        return;
      }
      setView(view() === "settings" ? settingsFrom() : "calendar");
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
    // Nothing reaches the rest of the panel while the gate is up. Every key
    // below this either navigates a calendar that is not showing or opens a view
    // that is not reachable - Enter set the view to `day`, invisibly, so picking
    // a city afterwards opened the app on a day detail nobody asked for.
    //
    // The gate's own field keeps its keys: this handler is on the panel, and
    // typing into an input never reaches it.
    if (!locationIsSet(props.boot)) return;

    if (event.metaKey && event.key === ",") {
      event.preventDefault();
      batch(() => {
        setSettingsFrom(view());
        setView("settings");
        setSection("root");
      });
      return;
    }
    if (event.metaKey || event.ctrlKey || event.altKey) return;

    if (event.key === "Escape") {
      event.preventDefault();
      // Innermost first. The picker is over the calendar, so Escape dismisses
      // it before it reaches the selection underneath - otherwise one press
      // cleared a ring the user could not see and left the picker open.
      if (jumping()) setJumping(false);
      // The chart has its own status item, so it is a peer of the calendar and
      // not a step inside it. Escaping to the calendar would also leave the back
      // end still believing the chart is showing, so its own item would then
      // hide the panel instead of returning to it.
      else if (view() === "chart") void ipc.closePanel();
      else if (view() !== "calendar") back();
      else if (selected()) setSelected(null);
      else void ipc.closePanel();
      return;
    }

    // The day view steps a day at a time, the same keys the calendar uses to
    // move the selection. Reading one day after another is most of what the day
    // view is for, and until now every step went back to the grid and picked
    // again - two moves and a change of view to see tomorrow.
    if (view() === "day") {
      if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return;
      event.preventDefault();
      moveSelection(event.key === "ArrowLeft" ? -1 : 1);
      return;
    }

    if (view() !== "calendar") return;
    // The picker has the keyboard while it is open: an arrow that moved the
    // selection behind it would move a ring nobody can see, and Enter would
    // open a day from the month being navigated away from.
    if (jumping()) return;

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
      // The cell that carries the tab stop, which is not always the selection:
      // with nothing selected the grid puts the stop on today, and failing that
      // on the first day of the month being displayed. Opening `selected() ??
      // today()` instead meant tabbing into a grid scrolled two months forward
      // and pressing Enter opened *today*, under a header still naming the
      // month you had scrolled to.
      Enter: () => openDay(keyboardDay()),
      " ": () => openDay(keyboardDay()),
      t: jumpToToday,
      T: jumpToToday,
    };

    const handler = handlers[event.key];
    if (!handler) return;

    // Not when a control has the focus.
    //
    // This is a window listener with no target check, and it called
    // `preventDefault` before dispatching - which cancels the click a button
    // synthesises from Enter or Space. Tabbing to the gear and pressing Enter
    // opened a day instead of settings; the month title, whose only job is the
    // jump overlay, could not be activated from the keyboard at all. Both were
    // reachable by mouse only, and `Cmd+,` was the sole keyboard route to
    // settings.
    //
    // A button, a link or a field owns its own keys. The grid does not: its
    // cells are divs and the arrows are the panel's.
    const target = event.target;
    if (
      target instanceof HTMLElement &&
      target.closest("button, a, input, select, textarea")
    ) {
      return;
    }

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
  /**
   * The day Enter opens, which is the cell the grid puts its tab stop on.
   *
   * The same three cases `MonthCells` uses, in the same order: the selection,
   * else today if today is in the month being displayed, else that month's
   * first day. Opening `selected() ?? today()` instead meant that tabbing into
   * a grid scrolled two months away and pressing Enter opened today - a day the
   * grid was not showing, under a header naming a different month.
   */
  function keyboardDay(): DateKey {
    const chosen = selected();
    if (chosen) return chosen;
    const days = monthData()?.days.filter((day) => day.in_month);
    if (!days || days.length === 0) return today();
    const now = today();
    const inMonth = days.some((day) => sameDate(day.date, now));
    return inMonth ? now : days[0]!.date;
  }

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
  /**
   * A tray item was clicked. The payload is a graha's key, or `chart`.
   *
   * `chart` is not a subject: the Lagna Kundali has its own status item but no
   * calendar of its own, so it opens as a *view* and leaves the subject at the
   * moon, which is the one that is always present. Passing it through as a
   * subject sent `chart` to `graha_month`, which refused it - the panel opened
   * on an error every time its own item was clicked.
   */
  function open(next: string) {
    const division = vargaFromKey(next);
    const chart = division !== undefined;
    const now = todayIn(props.boot.location.zone);
    batch(() => {
      if (division) setVarga(division);
      setSubject(chart ? "chandra" : (next as GrahaKey));
      setToday(now);
      setAnchor(noonAnchor(now, timeZone()));
      setOffset(0);
      setVisibleDelta(0);
      setSelected(null);
      setView(chart ? "chart" : "calendar");
      // The chart is a reading of now, and `chartAt` only advances on its own
      // timer while the chart is the view. Without this, reopening it after any
      // gap fetched the moment it was last showing.
      if (chart) setChartAt(Date.now());
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

    const opened = listen<string>("chandra://open", (event) => {
      setShown(true);
      open(event.payload as string);
    });
    onCleanup(() => void opened.then((unlisten) => unlisten()));

    const hidden = listen("chandra://hide", () => setShown(false));
    onCleanup(() => void hidden.then((unlisten) => unlisten()));
  });

  /** Whether the calendar in force names months by the Moon. */
  const lunar = () => props.boot.settings.calendar.month_system !== "solar";

  // A memo, not a plain accessor. It reads the chart resource, which is replaced
  // every second while the chart animates, so as a plain accessor it notified
  // its readers once a second with a string identical to the one they already
  // had - and the header's ladder reset on every one of them. A memo compares
  // and stays quiet.
  const headerTitle = createMemo(() => {
    if (view() === "settings") return SECTION_TITLES[section()];
    // The reading, not the genre. Every other header in the panel names a value
    // - `Chandra · August 2026`, `Shukla Ashtami`, the settings section - and
    // this one named the feature, which is the one thing on the surface a second
    // glance cannot recover from. The feature keeps its name in its settings
    // section and its tray tooltip.
    if (view() === "chart") {
      // `chart.latest`, not `chart()`: an errored resource re-throws from its
      // accessor, and this runs inside the header's title, so a failed chart
      // fetch threw out of render and took the whole view with it - including
      // the error block written to report exactly that.
      const rising = chart.error ? undefined : chart.latest?.lagna.name;
      if (!rising) return "";
      // The division first, then the reading. With sixteen charts and several of
      // them in the menu bar at once, a window that says only what is rising
      // does not say which chart it is - and the two can differ: D1 and D9 had
      // different rising signs in the same minute the first time this was
      // opened, which is the whole point of reading them together.
      //
      // `Rashi` is named too, rather than left implied for D1. Sixteen titles
      // that name their chart and one that does not would make the odd one out
      // look like a bug.
      const division = props.boot.vargas.find((v) => v.key === varga())?.name;
      return division ? `${division} · ${rising} Lagna` : `${rising} Lagna`;
    }
    // A lunar day is called by its tithi, so that is what the header says, and
    // the civil date it also has moves to the line below. In solar mode the
    // western date *is* the name, so the header is left empty and `Header`
    // falls back to it.
    //
    // The mode is tested explicitly. It used to be inferred from the panchanga
    // being absent, which was true only while `DayPanchanga` was an `Option`
    // that solar mode left `None`. Making it non-optional removed the inference
    // without removing the code that depended on it, so every mode got a tithi
    // for a title - and because the date line prints the civil date only in
    // lunar mode, a solar day ended up naming itself `Krishna Shashthi` with no
    // date anywhere on the surface.
    if (view() === "day") {
      if (!lunar()) return "";
      const panchanga = detail()?.panchanga;
      if (!panchanga) return "";
      const span =
        panchanga.tithis.find((tithi) => tithi.prevailing) ??
        panchanga.tithis[0];
      if (!span) return "";
      return `${span.paksha === "shukla" ? "Shukla" : "Krishna"} ${span.name}`;
    }
    return monthAt(visibleDelta())?.label ?? "";
  });

  // The window is sized by the back end from the same number; this scales what
  // is drawn inside it.
  createEffect(() => {
    document.documentElement.style.setProperty(
      "--panel-scale",
      String(props.boot.settings.appearance.scale),
    );
  });

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
        {/* While the gate is up the header carries the app's name and nothing
            else: there is no month to title, and a settings gear would offer a
            way round the one thing the app is insisting on. */}
        <Header
          division={Number(varga().slice(1))}
          subject={subject()}
          subjectName={grahaInfo()?.name ?? "Chandra"}
          info={grahaInfo()}
          snapshot={snapshot()}
          southern={props.boot.location.latitude < 0}
          title={headerTitle()}
          adhika={monthAt(visibleDelta())?.adhika ?? false}
          selected={selected()}
          view={view()}
          onBack={back}
          onSettings={() =>
            batch(() => {
              setJumping(false);
              // Where to return to. Only the keyboard path recorded this, so
              // opening settings from the chart with the gear and pressing back
              // landed on the moon calendar.
              setSettingsFrom(view());
              setView("settings");
              setSection("root");
            })
          }
          jumping={jumping()}
          onJump={() => setJumping((open) => !open)}
          gated={!locationIsSet(props.boot)}
        />

        <div class="region">
          {/* Before anything else, and not dismissible. Every figure below this
              point is computed from where the observer stands, and until that
              is known they would all be a guess dressed as a reading. D-029. */}
          <Show when={!locationIsSet(props.boot)}>
            <LocationGate
              boot={props.boot}
              apply={(next) => void applySettings(next)}
              error={error()}
            />
          </Show>

          {/* Every view renders the failure it can cause. The signal used to
              reach only the day detail, so a month that failed to load showed
              nothing at all in the calendar, and a settings save that failed
              fired while the user was by definition in settings - which made
              ERROR_TEXT.SETTINGS unreachable text. */}
          <Show when={locationIsSet(props.boot) && view() === "calendar"}>
            <Show
              when={error()}
              fallback={
                <>
                  {/* Keyed on the open counter: a fresh scroller per open. */}
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
                        onCommit={step}
                        onVisibleChange={setVisibleDelta}
                        scale={props.boot.settings.appearance.scale}
                        ingress={props.boot.settings.calendar.ingress}
                      />
                    )}
                  </Show>

                  {/* The grid says it too. The note used to be gated on the day
                    payload, so scrolling past 1800 drew 42 cells of numerals
                    from the analytic fallback in silence and only said so if a
                    day was opened - which is the reading D-006 exists to
                    prevent being passed off as exact. */}
                  <Show when={monthAt(visibleDelta())?.source === "moshier"}>
                    <p class="detail__provenance">
                      Outside 1800–2399. Times here are approximate, by about a
                      second.
                    </p>
                  </Show>
                </>
              }
            >
              {/* The month in the window is the one that failed - the grid has
                  nothing left to stay live for, so the error takes its place
                  (DESIGN 9.3). */}
              {(problem) => (
                <ErrorBlock code={problem().code} message={problem().message} />
              )}
            </Show>
          </Show>

          <Show when={locationIsSet(props.boot) && view() === "day"}>
            <DayDetail
              detail={detail()}
              events={selectedEvents()}
              context={{ timeZone: timeZone() }}
              isToday={sameDate(selected(), today())}
              lunar={lunar()}
              error={error()}
              // A Solid resource keeps its previous value while it refetches,
              // so stepping a day left yesterday's tithi, rise, set and
              // muhurtas on screen for the length of the round trip - under
              // today's arrows, presented as current. Said, rather than hidden
              // behind a blank the previous code was written to avoid.
              stale={detail.loading}
              onStep={(delta) => moveSelection(delta)}
            />
          </Show>

          <Show when={locationIsSet(props.boot) && view() === "chart"}>
            {/* `chart.latest`, not `chart()`, and not while it is errored. A
                Solid resource re-throws from its accessor once it has failed,
                so reading it here threw past the `error` prop below rather than
                filling it in. `latest` also keeps the previous reading on
                screen while the next is fetched, so the refresh does not blank
                the chart for a frame. */}
            <ChartView
              chart={chart.error ? undefined : chart.latest}
              error={chart.error ? toError(chart.error) : undefined}
              format={props.boot.settings.chart.format}
              numbered={props.boot.settings.chart.numbered}
              timeZone={timeZone()}
              animate={animating()}
              grid={props.boot.settings.chart.grid}
              sky={props.boot.settings.chart.sky}
            />
          </Show>

          <Show when={locationIsSet(props.boot) && view() === "settings"}>
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

          {/* Over the region, not in place of a view: what the strip is showing
              is the thing being changed, so it stays behind the picker rather
              than being replaced by it. Keyed on the cursor, so an overlay
              reopened after a jump starts on the year it landed in. */}
          <Show when={jumping() && view() === "calendar"}>
            <MonthJump
              anchor={anchor()}
              offset={offset()}
              firstWeekday={firstWeekday}
              context={context()}
              onJump={(target) =>
                batch(() => {
                  setJumping(false);
                  // Absolute, not a step: the offsets the index returns are
                  // against the anchor it was built for, which is this one.
                  setOffset(target);
                  // The selection belonged to the month being left. Kept, it
                  // would put the ring on a date the new month may not hold.
                  setSelected(null);
                })
              }
              onClose={() => setJumping(false)}
            />
          </Show>
        </div>
      </div>
    </div>
  );
}

/** Which list a settings section is reached from. */
const SETTINGS_PARENT: Partial<Record<SettingsSection, SettingsSection>> = {
  astrology: "advanced",
  panchanga: "advanced",
  ingress: "advanced",
  compartments: "advanced",
  motion: "advanced",
};

/** The division a tray key names, or `undefined` if the key is a graha's.
 *
 *  `chart:d9`. Prefixed so the kind is readable without a lookup table, and so a
 *  bare `d9` can never be mistaken for a graha key. */
function vargaFromKey(key: string): VargaKey | undefined {
  const division = key.startsWith("chart:") ? key.slice("chart:".length) : null;
  return division ? (division as VargaKey) : undefined;
}

/** Whether a subject key names a chart rather than a graha.
 *
 *  A predicate rather than a boolean, so the compiler narrows the other branch
 *  to `GrahaKey` and the `as GrahaKey` casts beside each call go away. Those
 *  casts were load-bearing: `Bootstrap.subject` was declared `GrahaKey |
 *  "chart"` while the back end has only ever sent `chart:{varga}`, and a cast
 *  is exactly what hides a union that does not describe the values. */
function isChartSubject(
  key: Bootstrap["subject"],
): key is `chart:${VargaKey}` {
  return key.startsWith("chart:");
}

function toError(thrown: unknown): { code: string; message: string } {
  if (isAppError(thrown)) return { code: thrown.code, message: thrown.message };
  return { code: "ENGINE", message: String(thrown) };
}
