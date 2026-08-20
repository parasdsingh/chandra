/**
 * Month chooser.
 *
 * Sized 4 x 3 so it fits the grid region exactly. That is the whole reason it is
 * a grid and not a scrolling list: the panel height must not change when it
 * opens (docs/DESIGN.md 5.7).
 */

import type { JSX } from "solid-js";
import { For } from "solid-js";

import { formatMonthShort } from "../lib/format";

interface Props {
  year: number;
  month: number;
  today: { year: number; month: number };
  onPick: (year: number, month: number) => void;
  onStepYear: (delta: number) => void;
}

export function MonthPicker(props: Props): JSX.Element {
  return (
    <div class="picker">
      <div class="picker__year">
        <button
          class="header__step"
          onClick={() => props.onStepYear(-1)}
          aria-label="Previous year"
        >
          ‹
        </button>
        <span class="picker__year-value">{props.year}</span>
        <button
          class="header__step"
          onClick={() => props.onStepYear(1)}
          aria-label="Next year"
        >
          ›
        </button>
      </div>

      <div class="picker__grid" role="grid" aria-label="Month">
        <For each={Array.from({ length: 12 }, (_, index) => index + 1)}>
          {(month) => (
            <button
              class="picker__cell"
              classList={{
                "is-current": props.year === props.year && month === props.month,
                "is-now":
                  props.year === props.today.year && month === props.today.month,
              }}
              role="gridcell"
              onClick={() => props.onPick(props.year, month)}
            >
              {formatMonthShort(month)}
            </button>
          )}
        </For>
      </div>
    </div>
  );
}
