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
  /** Fixed for the life of the page: the panel navigates on every open. */
  subject: GrahaKey;
  onSettingsApplied: (next: Bootstrap) => void;
}

export function Panel(props: Props): JSX.Element {
  const timeZone = () => props.boot.location.zone;
  const today = () => todayIn(timeZone());

  // Navigation is by anchor instant plus offset, because a lunar month has no
  // year-and-number to step through.
  const [anchor, setAnchor] = createSignal(noonAnchor(todayIn(props.boot.location.zone)));
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

  function monthKey(delta: number): string {
    return [
      props.subject,
      props.boot.settings.calendar.month_system,
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
      subject: props.subject,
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

  const [snapshot] = createResource(async () => {
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
    return date && view() === "day" ? { subject: props.subject, date } : null;
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
    props.boot.grahas.find((graha) => graha.key === props.subject),
  );

  const selectedEvents = createMemo(() => {
    const date = selected();
    const data = monthData();
    if (!date || !data || props.subject === "chandra") return [];
    return (data as GrahaMonth).events.filter(
      (event) =>
        event.date.year === date.year &&
        event.date.month === date.month &&
        event.date.day === date.day,
    );
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
        setAnchor(noonAnchor(next));
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
      setAnchor(noonAnchor(now));
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
      Home: () => selectEdge(0),
      End: () => selectEdge(-1),
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

  function selectEdge(index: number) {
    const days = monthData()?.days;
    if (!days || days.length === 0) return;
    setSelected(days.at(index)?.date ?? null);
  }

  let root: HTMLDivElement | undefined;
  onMount(() => {
    root?.focus();
    const listener = (event: KeyboardEvent) => onKeyDown(event);
    window.addEventListener("keydown", listener);
    onCleanup(() => window.removeEventListener("keydown", listener));
  });

  // Nothing to reset: the backend navigates the page on every open, so each
  // open starts from a fresh component tree on the calendar, at today.

  const headerTitle = () => {
    if (view() === "settings") return SECTION_TITLES[section()];
    if (view() === "day") return "";
    return monthAt(visibleDelta())?.label ?? "";
  };

  return (
    <div class="panel-frame">
      <div class="panel" ref={root} tabindex="-1" role="dialog" aria-label="Chandra">
        <Header
          subject={props.subject}
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
              kind={props.subject === "chandra" ? "moon" : "graha"}
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
