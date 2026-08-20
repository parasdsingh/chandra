/**
 * The scrolling month strip.
 *
 * Three months are stacked and moved together, so a wheel or a drag carries the
 * calendar with it continuously rather than jumping a month at a time. Releasing
 * settles to whichever month is closer. That is the only navigation that works
 * the same for a lunar month, where stepping goes from one syzygy to the next
 * and there is no numbered sequence to page through.
 */

import type { JSX } from "solid-js";
import { batch, createSignal, For, onCleanup, Show } from "solid-js";

import type { DateKey, GrahaInfo, GrahaMonth, MoonMonth } from "../ipc/types";
import { buildGrid } from "../lib/calendar";
import { MonthCells, WeekdayRow } from "./MonthGrid";

/** Height of one month of cells: six rows of 40px. */
const MONTH_HEIGHT = 240;

/** How far the strip must travel before releasing settles onto a new month. */
const COMMIT_DISTANCE = MONTH_HEIGHT * 0.28;

/** Idle time after the last wheel event that counts as the end of a gesture. */
const WHEEL_END_MS = 110;

/** Resistance at the ends of the loaded range, so the strip cannot be flung past it. */
const OVERSCROLL_LIMIT = MONTH_HEIGHT;

interface Props {
  firstWeekday: number;
  previous: MoonMonth | GrahaMonth | undefined;
  current: MoonMonth | GrahaMonth | undefined;
  next: MoonMonth | GrahaMonth | undefined;
  kind: "moon" | "graha";
  info: GrahaInfo | undefined;
  selected: DateKey | null;
  today: DateKey;
  southern: boolean;
  onSelect: (date: DateKey) => void;
  /** Called once the strip has settled onto a neighbouring month. */
  onCommit: (delta: number) => void;
}

export function CalendarScroller(props: Props): JSX.Element {
  // Offset from the resting position, where the current month fills the window.
  const [shift, setShift] = createSignal(0);
  const [settling, setSettling] = createSignal(false);

  let wheelTimer: number | undefined;
  let dragging = false;
  let dragOrigin = 0;
  let dragShift = 0;
  let moved = false;

  onCleanup(() => window.clearTimeout(wheelTimer));

  function clamp(value: number): number {
    return Math.max(-OVERSCROLL_LIMIT, Math.min(OVERSCROLL_LIMIT, value));
  }

  /** Ends a gesture: either land on a neighbour, or return to rest. */
  function settle() {
    const travelled = shift();
    if (Math.abs(travelled) < COMMIT_DISTANCE) {
      setSettling(true);
      setShift(0);
      return;
    }

    // Moving the strip up reveals the following month.
    const delta = travelled < 0 ? 1 : -1;
    setSettling(true);
    setShift(delta > 0 ? -MONTH_HEIGHT : MONTH_HEIGHT);
  }

  /**
   * Called when the settle animation finishes.
   *
   * Committing here rather than on release is what makes the change seamless:
   * the neighbour has already slid into place, so swapping it to be the centre
   * month and resetting the offset in the same frame changes nothing on screen.
   */
  function onTransitionEnd() {
    if (!settling()) return;
    const landed = shift();
    batch(() => {
      setSettling(false);
      if (landed !== 0) {
        setShift(0);
        props.onCommit(landed < 0 ? 1 : -1);
      }
    });
  }

  function onWheel(event: WheelEvent) {
    event.preventDefault();
    if (settling()) return;

    setShift((current) => clamp(current - event.deltaY));

    window.clearTimeout(wheelTimer);
    wheelTimer = window.setTimeout(settle, WHEEL_END_MS);
  }

  function onPointerDown(event: PointerEvent) {
    if (event.button !== 0 || settling()) return;
    dragging = true;
    moved = false;
    dragOrigin = event.clientY;
    dragShift = shift();
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  }

  function onPointerMove(event: PointerEvent) {
    if (!dragging) return;
    const travel = event.clientY - dragOrigin;
    if (Math.abs(travel) > 3) moved = true;
    setShift(clamp(dragShift + travel));
  }

  function onPointerUp(event: PointerEvent) {
    if (!dragging) return;
    dragging = false;
    (event.currentTarget as HTMLElement).releasePointerCapture(event.pointerId);
    settle();
  }

  const months = () => [
    { month: props.previous, offset: -MONTH_HEIGHT },
    { month: props.current, offset: 0 },
    { month: props.next, offset: MONTH_HEIGHT },
  ];

  return (
    <div class="grid-region">
      <WeekdayRow firstWeekday={props.firstWeekday} />

      <div
        class="scroller"
        onWheel={onWheel}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
        onPointerCancel={onPointerUp}
      >
        <div
          class="scroller__strip"
          classList={{ "is-settling": settling() }}
          style={{ transform: `translateY(${shift()}px)` }}
          onTransitionEnd={onTransitionEnd}
        >
          <For each={months()}>
            {(entry) => (
              <div class="scroller__month" style={{ top: `${entry.offset}px` }}>
                <Show when={entry.month}>
                  {(month) => (
                    <MonthCells
                      grid={buildGrid(
                        month().days.map((day) => day.date),
                        props.firstWeekday,
                      )}
                      firstWeekday={props.firstWeekday}
                      month={month()}
                      kind={props.kind}
                      info={props.info}
                      selected={props.selected}
                      today={props.today}
                      southern={props.southern}
                      direction={0}
                      onSelect={(date) => {
                        // A drag that ends over a cell must not also select it.
                        if (moved) return;
                        props.onSelect(date);
                      }}
                    />
                  )}
                </Show>
              </div>
            )}
          </For>
        </div>
      </div>
    </div>
  );
}
