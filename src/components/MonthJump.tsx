/**
 * The pointer route to a year.
 *
 * A wheel moves one month per notch, so before this the only way to 2140 was
 * the keyboard - and the panel prints a precision note outside 1800-2399, which
 * is a promise that going there is possible.
 *
 * The months come from the back end rather than being counted here. A Vikram
 * Samvat year holds twelve months or thirteen, and which it is depends on
 * whether a lunation fitted inside one solar rashi. Twelve cells laid out on
 * faith would put the adhika masa in the wrong place, or lose it.
 */

import type { JSX } from "solid-js";
import { createResource, createSignal, For, Show } from "solid-js";

import * as ipc from "../ipc";
import type { MonthIndex } from "../ipc/types";

interface Props {
  /** The cursor this overlay opened over. */
  anchor: number;
  offset: number;
  firstWeekday: number;
  /** Re-fetches when the configuration behind a month changes under it. */
  context: string;
  onJump: (offset: number) => void;
  onClose: () => void;
}

export function MonthJump(props: Props): JSX.Element {
  // Which year is on show. Starts at the month behind the overlay and moves by
  // the offsets the index reports, never by adding twelve.
  const [probe, setProbe] = createSignal(props.offset);

  const [index] = createResource(
    () => ({ probe: probe(), context: props.context, anchor: props.anchor }),
    async (key): Promise<MonthIndex | undefined> => {
      try {
        return await ipc.monthIndex(key.anchor, key.probe, props.firstWeekday);
      } catch {
        // The year is unreachable - past the ephemeris, or the engine is busy
        // failing. The overlay keeps the year it had rather than emptying, and
        // the arrow that led here simply did nothing.
        return index.latest;
      }
    },
  );

  return (
    // The backdrop closes it. The strip behind is visible through it and reads
    // as the thing being changed, so clicking it means "not that one" rather
    // than "take me to that month" - a click that both dismissed and navigated
    // would fire on every mis-aimed press.
    <div class="jump-scrim" onClick={props.onClose}>
      <div
        class="jump"
        role="dialog"
        aria-label="Jump to a month"
        onClick={(event) => event.stopPropagation()}
      >
        <div class="jump__bar">
          <button
            class="jump__step"
            aria-label="Previous year"
            onClick={() =>
              setProbe(index.latest?.previous_year ?? probe() - 12)
            }
          >
            <Arrow direction="left" />
          </button>

          {/* Polite rather than assertive: paging a year is the user's own doing
            and does not need to interrupt them, but the year is the only thing
            that changes and a reader would otherwise have to go looking. */}
          <span class="jump__year" aria-live="polite">
            {index.latest?.year ?? ""}
          </span>

          <button
            class="jump__step"
            aria-label="Next year"
            onClick={() => setProbe(index.latest?.next_year ?? probe() + 12)}
          >
            <Arrow direction="right" />
          </button>
        </div>

        {/* No `role="list"` and no `role="listitem"`. They were here to say the
            months are a set, and they cost more than they said: `listitem` on a
            `<button>` replaces the button role outright, so twelve or thirteen
            activatable controls were announced as list items - things to read
            past rather than things to press. A group with a label says the same
            thing and takes nothing away. */}
        <div class="jump__months" role="group" aria-label="Months">
          <For each={index.latest?.months ?? []}>
            {(month) => (
              <button
                class="jump__month"
                classList={{ "is-current": month.offset === props.offset }}
                // The pressed state, not a colour: which month is in view behind
                // the overlay is a fact a reader needs whether or not the accent
                // reaches them.
                aria-current={
                  month.offset === props.offset ? "true" : undefined
                }
                onClick={() => props.onJump(month.offset)}
              >
                {/* `Adhika` is a qualifier on the name, set apart here for the
                  same reason the header sets it apart: run together it reads as
                  a thirteenth month name of its own. */}
                <Show when={month.adhika}>
                  <span class="jump__qualifier">Adhika </span>
                </Show>
                {month.adhika
                  ? month.name.replace(/^Adhika\s+/, "")
                  : month.name}
              </button>
            )}
          </For>
        </div>
      </div>
    </div>
  );
}

function Arrow(props: { direction: "left" | "right" }): JSX.Element {
  return (
    <svg viewBox="0 0 16 16" width="12" height="12" aria-hidden="true">
      <path
        d={
          props.direction === "left"
            ? "M 10 3 L 5 8 L 10 13"
            : "M 6 3 L 11 8 L 6 13"
        }
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
  );
}
