/**
 * The scrolling month strip.
 *
 * Three months are stacked and moved together, so a wheel or a drag carries the
 * calendar with it continuously rather than jumping a month at a time. That is
 * the only navigation that works the same for a lunar month, where stepping goes
 * from one syzygy to the next and there is no numbered sequence to page through.
 *
 * The settle is driven by requestAnimationFrame rather than a CSS transition.
 * A transition ends by firing `transitionend`, which does not arrive when the
 * target equals the current value, and which also fires for any descendant's
 * transition because it bubbles. Both left the strip wedged: a plain click set
 * the strip animating to an offset it was already at, no event ever came, and
 * every later gesture was refused. A frame loop finishes because it counts
 * frames, not because an event happens to arrive.
 *
 * Input is never refused. A wheel or a drag during the settle takes the strip
 * over from wherever it has reached. Trackpad momentum keeps delivering wheel
 * events long after the fingers lift, so refusing them mid-settle is what made
 * a flick feel like it had jammed.
 *
 * Every mover states a distance to travel, never a position to be at, and the
 * month changes the moment the strip has travelled a whole month rather than
 * when an animation reports it has finished. Those two together are what make
 * interruption safe: a gesture that cuts a settle short keeps whatever the
 * settle had already covered, because there is no result still owed. Holding the
 * change until the animation ended meant an interrupted settle threw its month
 * away, and four quick flicks moved three months.
 */

import type { JSX } from "solid-js";
import { batch, createSignal, Index, onCleanup, Show } from "solid-js";

import type { DateKey, GrahaInfo, GrahaMonth, MoonMonth } from "../ipc/types";
import { MonthCells, WeekdayRow } from "./MonthGrid";

/** Height of one month of cells: six rows of 40px. */
const MONTH_HEIGHT = 240;

/** How far the strip must travel before releasing settles onto a new month. */
const COMMIT_DISTANCE = MONTH_HEIGHT * 0.28;

/** Idle time after the last wheel event that counts as the end of a gesture. */
const WHEEL_END_MS = 110;

/**
 * Furthest a single move may carry the strip.
 *
 * WebKit coalesces wheel events under load, so a whole flick can arrive as one
 * delta, and running the calendar several months on one event is not a gesture.
 * Exactly three months, so that banking always drains the displacement below one
 * month: bounding the number of banks instead left a residual of a full month
 * whenever the limit was reached, and a strip sitting exactly at its neighbour's
 * resting position has a settle distance of zero. `animate(0)` returns without
 * doing anything, so nothing ever brought it back and the calendar stayed a
 * month off its own offset.
 */
const MAX_TRAVEL = MONTH_HEIGHT * 3;

/**
 * Release speed, in pixels per millisecond, that commits regardless of distance.
 *
 * A flick is a statement of intent: it should change the month even though the
 * fingers left before the strip had travelled far.
 */
const FLICK_SPEED = 0.45;

/** Window over which release speed is measured. Long enough to be stable. */
const VELOCITY_WINDOW_MS = 90;

/**
 * How stale a drag's last sample may be and still count as a release.
 *
 * A finger held still before the button comes up did not flick, however fast it
 * had been moving a moment earlier. A wheel needs no such test and is given
 * none: it has no separate stop to detect, because the idle timer that ends the
 * gesture *is* the stop. Timing a wheel release against a wall clock instead
 * made it a race with the re-render a month change triggers - the timer fired
 * 190ms after the last event rather than 110, every release read as motionless,
 * and the flick never carried.
 */
const DRAG_RELEASE_MS = 50;

const SETTLE_MIN_MS = 140;
const SETTLE_MAX_MS = 300;

/** Reduced motion still moves - it just arrives sooner. */
const SETTLE_REDUCED_MS = 80;

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
  /**
   * What the panel is drawn at.
   *
   * The strip is composed at 1.0 and the whole panel is scaled by a transform,
   * so a pointer moves `scale` screen pixels for every composed pixel the strip
   * should travel. Pointer and wheel deltas arrive in screen pixels and are
   * divided back before they are used, or at the largest size the calendar
   * slides a third further than the finger that pushed it.
   */
  scale: number;
  /**
   * Which month now fills most of the window, before it has been committed.
   *
   * The header reads its title from this, so the title changes when the new
   * month becomes the one being looked at rather than a beat later when the
   * strip stops moving.
   */
  onVisibleChange: (delta: number) => void;
}

interface Sample {
  at: number;
  travelled: number;
}

export function CalendarScroller(props: Props): JSX.Element {
  // Offset from the resting position, where the current month fills the window.
  const [shift, setShift] = createSignal(0);

  let wheelTimer: number | undefined;
  let frame: number | undefined;
  let dragging = false;
  let dragFrom = 0;
  let dragTravel = 0;
  let moved = false;
  let visible = 0;
  /**
   * Total ground covered, ignoring every rebase.
   *
   * Release speed is measured against this rather than against the strip's
   * offset, which jumps by a whole month each time one is banked and would read
   * as an impossible flick in the opposite direction.
   */
  let odometer = 0;
  let samples: Sample[] = [];

  onCleanup(() => {
    window.clearTimeout(wheelTimer);
    if (frame !== undefined) cancelAnimationFrame(frame);
  });

  const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)");

  /**
   * Moves the strip by `travel` pixels.
   *
   * A whole month of travel is banked immediately: the neighbour is promoted to
   * be the centre month and the offset is reduced by that month's height, in one
   * batch. On screen that is a no-op, because the neighbour is already drawn
   * exactly where the centre month is about to be. Nothing is left owed, so an
   * interrupted gesture cannot lose a month.
   *
   * Every path that moves the strip goes through here, so the header can never
   * disagree with what fills the window.
   */
  function move(travel: number) {
    const previous = shift();

    // Bounded before the banking, not after it, so the banking always finishes
    // and the strip is never left a whole month from rest.
    let value = Math.max(-MAX_TRAVEL, Math.min(MAX_TRAVEL, previous + travel));
    let banked = 0;

    while (Math.abs(value) >= MONTH_HEIGHT) {
      const direction = value < 0 ? 1 : -1;
      value += direction * MONTH_HEIGHT;
      banked += direction;
    }
    odometer += value - previous - banked * MONTH_HEIGHT;

    // Halfway: past this the neighbour occupies more of the window than the
    // month whose name is being shown.
    const showing =
      value <= -MONTH_HEIGHT / 2 ? 1 : value >= MONTH_HEIGHT / 2 ? -1 : 0;

    batch(() => {
      setShift(value);
      if (banked !== 0) props.onCommit(banked);
      if (showing !== visible) {
        visible = showing;
        props.onVisibleChange(showing);
      }
    });
  }

  function record() {
    const now = performance.now();
    samples.push({ at: now, travelled: odometer });
    // Never below two. WebKit coalesces wheel events under load - a whole
    // gesture can arrive as one delta more than a window apart from the last -
    // and trimming to a single sample left every such release reading as
    // motionless, so the flick that should have carried the strip on never
    // fired. Two samples far apart still give an honest speed.
    while (samples.length > 2 && now - samples[0]!.at > VELOCITY_WINDOW_MS) {
      samples.shift();
    }
  }

  /**
   * Pixels per millisecond over the sampling window; negative is upward.
   *
   * Zero once the last sample is older than `maxAge`: a gesture that came to a
   * stop before it ended was not a flick, however fast it had been moving. An
   * absent `maxAge` skips that test, for a gesture whose end is detected by the
   * input stopping rather than reported by the input itself.
   */
  function velocity(maxAge: number | undefined): number {
    if (samples.length < 2) return 0;
    const first = samples[0]!;
    const last = samples[samples.length - 1]!;
    const elapsed = last.at - first.at;
    if (elapsed <= 0) return 0;
    if (maxAge !== undefined && performance.now() - last.at > maxAge) return 0;
    return (last.travelled - first.travelled) / elapsed;
  }

  function stopAnimation() {
    if (frame !== undefined) {
      cancelAnimationFrame(frame);
      frame = undefined;
    }
  }

  /** Ends a gesture: either land on a neighbour, or return to rest. */
  function settle(maxAge: number | undefined) {
    const travelled = shift();
    const speed = velocity(maxAge);

    // Moving the strip up reveals the following month.
    const flicked =
      Math.abs(speed) >= FLICK_SPEED &&
      travelled !== 0 &&
      Math.sign(speed) === Math.sign(travelled);

    const onward = flicked || Math.abs(travelled) >= COMMIT_DISTANCE;

    // Either the rest of the way to the neighbour, or back to where it started.
    animate(onward ? (travelled < 0 ? -MONTH_HEIGHT : MONTH_HEIGHT) - travelled : -travelled);
  }

  /**
   * Carries the strip a further `distance` pixels, decelerating.
   *
   * Frame by frame it hands `move` the ground covered since the last frame, so
   * a month banked part way through is simply gone from the residual and the
   * remaining frames carry on from there.
   */
  function animate(distance: number) {
    stopAnimation();
    if (Math.abs(distance) < 0.5) {
      samples = [];
      return;
    }

    const duration = reducedMotion.matches
      ? SETTLE_REDUCED_MS
      : Math.max(
          SETTLE_MIN_MS,
          Math.min(SETTLE_MAX_MS, (Math.abs(distance) / MONTH_HEIGHT) * SETTLE_MAX_MS),
        );
    const started = performance.now();
    let covered = 0;

    const step = (now: number) => {
      const t = Math.min(1, (now - started) / duration);
      const reached = distance * ease(t);
      move(reached - covered);
      covered = reached;

      if (t < 1) {
        frame = requestAnimationFrame(step);
        return;
      }
      frame = undefined;
      samples = [];
    };

    frame = requestAnimationFrame(step);
  }

  function onWheel(event: WheelEvent) {
    event.preventDefault();
    stopAnimation();

    move(-event.deltaY / props.scale);
    record();

    window.clearTimeout(wheelTimer);
    wheelTimer = window.setTimeout(() => settle(undefined), WHEEL_END_MS);
  }

  function onPointerDown(event: PointerEvent) {
    if (event.button !== 0) return;
    stopAnimation();
    window.clearTimeout(wheelTimer);

    dragging = true;
    moved = false;
    dragFrom = event.clientY;
    dragTravel = 0;
    samples = [];
    record();
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  }

  function onPointerMove(event: PointerEvent) {
    if (!dragging) return;
    const step = (event.clientY - dragFrom) / props.scale;
    dragFrom = event.clientY;
    dragTravel += step;
    if (Math.abs(dragTravel) > 3) moved = true;
    move(step);
    record();
  }

  function onPointerUp(event: PointerEvent) {
    if (!dragging) return;
    dragging = false;
    (event.currentTarget as HTMLElement).releasePointerCapture(event.pointerId);
    // A press with no travel is a click on a day, not a gesture. Settling it
    // would be a no-op animation whose only effect is to swallow the next input.
    if (moved) settle(DRAG_RELEASE_MS);
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
          style={{ transform: `translateY(${shift()}px)` }}
        >
          {/* Index, not For. `months()` builds three fresh objects every read
              and For keys by reference, so each commit disposed and rebuilt all
              126 cells in the middle of a gesture. Index keys by position,
              which is what these three slots are. */}
          <Index each={months()}>
            {(entry) => (
              <div class="scroller__month" style={{ top: `${entry().offset}px` }}>
                <Show when={entry().month}>
                  {(month) => (
                    <MonthCells
                      firstWeekday={props.firstWeekday}
                      month={month()}
                      kind={props.kind}
                      info={props.info}
                      selected={props.selected}
                      today={props.today}
                      southern={props.southern}
                      active={entry().offset === 0}
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
          </Index>
        </div>
      </div>
    </div>
  );
}

/** Cubic ease-out: fastest at the moment the fingers leave, then decelerating. */
function ease(t: number): number {
  return 1 - Math.pow(1 - t, 3);
}
