/** The popover panel: header, grid or month picker, and the expanded detail. */

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
  Snapshot,
} from "../ipc/types";
import { isAppError } from "../ipc/types";
import {
  addDays,
  addMonths,
  buildGrid,
  daysInMonth,
  sameDate,
  todayIn,
} from "../lib/calendar";
import { localeFirstWeekday } from "../lib/format";
import { DayDetail } from "./DayDetail";
import { Header } from "./Header";
import { MonthGrid } from "./MonthGrid";
import { MonthPicker } from "./MonthPicker";

interface Props {
  boot: Bootstrap;
  subject: GrahaKey;
}

export function Panel(props: Props): JSX.Element {
  const timeZone = () => props.boot.location.zone;
  const today = () => todayIn(timeZone());

  const [year, setYear] = createSignal(today().year);
  const [month, setMonth] = createSignal(today().month);
  const [selected, setSelected] = createSignal<DateKey | null>(null);
  const [pickerOpen, setPickerOpen] = createSignal(false);
  const [direction, setDirection] = createSignal(0);
  const [error, setError] = createSignal<{ code: string; message: string }>();

  const firstWeekday = localeFirstWeekday();

  const monthKey = createMemo(() => ({
    subject: props.subject,
    year: year(),
    month: month(),
  }));

  const [monthData] = createResource(monthKey, async (key) => {
    setError(undefined);
    try {
      return key.subject === "chandra"
        ? ((await ipc.moonMonth(key.year, key.month)) as MoonMonth)
        : ((await ipc.grahaMonth(key.subject, key.year, key.month)) as GrahaMonth);
    } catch (thrown) {
      setError(toError(thrown));
      return undefined;
    }
  });

  const [snapshot] = createResource(async () => {
    try {
      return (await ipc.snapshot(Date.now())) as Snapshot;
    } catch {
      // The header glyph is decoration; failing to draw it must not block the
      // grid, which is what the user opened the panel for.
      return undefined;
    }
  });

  const detailKey = createMemo(() => {
    const date = selected();
    return date ? { subject: props.subject, date } : null;
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

  const grid = createMemo(() =>
    buildGrid(year(), month(), monthData()?.leading_blanks ?? 0, firstWeekday),
  );

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

  function goToMonth(nextYear: number, nextMonth: number, travel: number) {
    batch(() => {
      setDirection(travel);
      setYear(nextYear);
      setMonth(nextMonth);
    });
  }

  function stepMonth(delta: number) {
    const next = addMonths(year(), month(), delta);
    goToMonth(next.year, next.month, delta);
  }

  /** Moves the selection, following it across a month boundary. */
  function moveSelection(deltaDays: number) {
    const from = selected() ?? today();
    const next = addDays(from, deltaDays);
    batch(() => {
      setSelected(next);
      if (next.year !== year() || next.month !== month()) {
        setDirection(deltaDays > 0 ? 1 : -1);
        setYear(next.year);
        setMonth(next.month);
      }
    });
  }

  function selectDay(date: DateKey) {
    if (date.year !== year() || date.month !== month()) {
      const travel =
        date.year * 12 + date.month > year() * 12 + month() ? 1 : -1;
      goToMonth(date.year, date.month, travel);
    }
    // Clicking the selected day again collapses the detail, so the same gesture
    // opens and closes it.
    setSelected((current) => (sameDate(current, date) ? null : date));
  }

  function jumpToToday() {
    const now = today();
    batch(() => {
      setDirection(0);
      setYear(now.year);
      setMonth(now.month);
      setSelected(now);
    });
  }

  function onKeyDown(event: KeyboardEvent) {
    if (event.metaKey && event.key === ",") {
      event.preventDefault();
      void ipc.openSettings();
      return;
    }
    if (event.metaKey && event.key.toLowerCase() === "w") {
      event.preventDefault();
      void ipc.closePanel();
      return;
    }
    if (event.metaKey || event.ctrlKey || event.altKey) return;

    const handlers: Record<string, () => void> = {
      ArrowLeft: () => moveSelection(-1),
      ArrowRight: () => moveSelection(1),
      ArrowUp: () => moveSelection(-7),
      ArrowDown: () => moveSelection(7),
      PageUp: () => stepYearOrMonth(event.shiftKey ? -12 : -1),
      PageDown: () => stepYearOrMonth(event.shiftKey ? 12 : 1),
      Home: () => setSelected({ year: year(), month: month(), day: 1 }),
      End: () =>
        setSelected({
          year: year(),
          month: month(),
          day: daysInMonth(year(), month()),
        }),
      Enter: () => toggleDetail(),
      " ": () => toggleDetail(),
      Escape: () => {
        if (pickerOpen()) setPickerOpen(false);
        else if (selected()) setSelected(null);
        else void ipc.closePanel();
      },
      t: jumpToToday,
      T: jumpToToday,
      m: () => setPickerOpen((open) => !open),
      M: () => setPickerOpen((open) => !open),
    };

    const handler = handlers[event.key];
    if (!handler) return;
    // Unbound keys are silently ignored: no beep, no shake.
    event.preventDefault();
    handler();
  }

  /**
   * Page keys keep the day of month, clamped to the target month's length, so
   * stepping from the 31st into a 30 day month lands on the 30th rather than
   * silently rolling into the next month.
   */
  function stepYearOrMonth(delta: number) {
    const next = addMonths(year(), month(), delta);
    const current = selected();
    goToMonth(next.year, next.month, delta > 0 ? 1 : -1);
    if (current) {
      setSelected({
        year: next.year,
        month: next.month,
        day: Math.min(current.day, daysInMonth(next.year, next.month)),
      });
    }
  }

  function toggleDetail() {
    setSelected((current) => (current ? null : today()));
  }

  let root: HTMLDivElement | undefined;
  onMount(() => {
    root?.focus();
    const listener = (event: KeyboardEvent) => onKeyDown(event);
    window.addEventListener("keydown", listener);
    onCleanup(() => window.removeEventListener("keydown", listener));
  });

  // Re-focus the panel whenever the subject changes, so the keyboard works
  // immediately after a tray click without a further click into the window.
  createEffect(() => {
    void props.subject;
    root?.focus();
  });

  return (
    <div class="panel-frame">
      <div
        class="panel"
        classList={{ "is-expanded": Boolean(selected()) }}
        ref={root}
        tabindex="-1"
        role="dialog"
        aria-label={`Chandra, ${props.subject}`}
      >
        <Header
          subject={props.subject}
          subjectName={grahaInfo()?.name ?? "Chandra"}
          info={grahaInfo()}
          snapshot={snapshot()}
          southern={props.boot.location.latitude < 0}
          year={year()}
          month={month()}
          pickerOpen={pickerOpen()}
          onTogglePicker={() => setPickerOpen((open) => !open)}
          onStep={stepMonth}
          onSettings={() => void ipc.openSettings()}
        />

        <Show
          when={!pickerOpen()}
          fallback={
            <MonthPicker
              year={year()}
              month={month()}
              today={today()}
              onPick={(pickedYear, pickedMonth) => {
                goToMonth(pickedYear, pickedMonth, 0);
                setPickerOpen(false);
              }}
              onStepYear={(delta) => setYear((value) => value + delta)}
            />
          }
        >
          <MonthGrid
            grid={grid()}
            firstWeekday={firstWeekday}
            month={monthData()}
            kind={props.subject === "chandra" ? "moon" : "graha"}
            selected={selected()}
            today={today()}
            southern={props.boot.location.latitude < 0}
            direction={direction()}
            onSelect={selectDay}
          />
        </Show>

        <Show when={selected()}>
          <div class="panel__divider" />
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
      </div>
    </div>
  );
}

function toError(thrown: unknown): { code: string; message: string } {
  if (isAppError(thrown)) return { code: thrown.code, message: thrown.message };
  return { code: "ENGINE", message: String(thrown) };
}
