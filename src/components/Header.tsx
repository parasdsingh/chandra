/** Panel header: subject glyph, month label, month steppers, settings. */

import type { JSX } from "solid-js";
import { createEffect, createSignal, on, Show } from "solid-js";

import type { GrahaInfo, GrahaKey, Snapshot } from "../ipc/types";
import { formatMonthShort, formatMonthYear } from "../lib/format";
import { GrahaGlyph } from "./GrahaGlyph";
import { PhaseGlyph } from "./PhaseGlyph";

interface Props {
  subject: GrahaKey;
  subjectName: string;
  info: GrahaInfo | undefined;
  snapshot: Snapshot | undefined;
  southern: boolean;
  year: number;
  month: number;
  pickerOpen: boolean;
  onTogglePicker: () => void;
  onStep: (delta: number) => void;
  onSettings: () => void;
}

export function Header(props: Props): JSX.Element {
  const full = () => `${props.subjectName} · ${formatMonthYear(props.year, props.month)}`;

  // Ladder from docs/DESIGN.md 5.2. No ellipsis: each step drops a whole word.
  const candidates = () => [
    full(),
    `${props.subjectName} · ${formatMonthShort(props.month)} ${props.year}`,
    `${formatMonthYear(props.year, props.month)}`,
    `${formatMonthShort(props.month)} ${props.year}`,
  ];

  let labelElement: HTMLButtonElement | undefined;
  const [level, setLevel] = createSignal(0);
  const fitted = () => candidates()[level()] ?? full();

  // Start again from the full label whenever what it says changes.
  createEffect(
    on(
      () => [props.subjectName, props.year, props.month],
      () => setLevel(0),
    ),
  );

  // Step down the ladder until the text stops overflowing its box.
  //
  // Measured from real layout rather than with a canvas: assigning the computed
  // font shorthand to a canvas context silently fails for `-apple-system`, and
  // the context keeps its 10px default, so every candidate appears to fit and
  // the label clips mid-character.
  createEffect(() => {
    const current = fitted();
    void current;
    const element = labelElement;
    if (!element) return;
    if (
      element.scrollWidth > element.clientWidth &&
      level() < candidates().length - 1
    ) {
      setLevel(level() + 1);
    }
  });

  return (
    <header class="header">
      <span class="header__glyph">
        <Show
          when={props.subject === "chandra" && props.snapshot}
          fallback={
            <Show when={props.info}>
              {(info) => <GrahaGlyph info={info()} size={16} />}
            </Show>
          }
        >
          {(snapshot) => (
            <PhaseGlyph
              illumination={snapshot().illumination}
              waxing={snapshot().is_waxing}
              southern={props.southern}
              size={16}
            />
          )}
        </Show>
      </span>

      <button
        class="header__label"
        ref={labelElement}
        onClick={props.onTogglePicker}
        aria-expanded={props.pickerOpen}
        aria-label={`${full()}. Choose a month`}
      >
        {fitted()}
      </button>

      <button
        class="header__step"
        onClick={() => props.onStep(-1)}
        aria-label="Previous month"
      >
        <Chevron direction="left" />
      </button>
      <button
        class="header__step"
        onClick={() => props.onStep(1)}
        aria-label="Next month"
      >
        <Chevron direction="right" />
      </button>
      <button class="header__step" onClick={props.onSettings} aria-label="Settings">
        <Gear />
      </button>
    </header>
  );
}

function Chevron(props: { direction: "left" | "right" }): JSX.Element {
  const d = () =>
    props.direction === "left" ? "M 14.5 5 L 9.5 12 L 14.5 19" : "M 9.5 5 L 14.5 12 L 9.5 19";
  return (
    <svg width="24" height="24" viewBox="0 0 24 24" aria-hidden="true">
      <path
        d={d()}
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
  );
}

function Gear(): JSX.Element {
  // Teeth start at the rim and are short and blunt. Long thin spokes standing
  // clear of the circle read as a sun, not a gear - which is what the first
  // version of this icon did.
  const RIM = 6.0;
  const TIP = 8.4;
  const teeth = Array.from({ length: 8 }, (_, index) => {
    const angle = (index * Math.PI) / 4 + Math.PI / 8;
    return {
      x1: 12 + Math.cos(angle) * RIM,
      y1: 12 + Math.sin(angle) * RIM,
      x2: 12 + Math.cos(angle) * TIP,
      y2: 12 + Math.sin(angle) * TIP,
    };
  });

  return (
    <svg width="24" height="24" viewBox="0 0 24 24" aria-hidden="true">
      {teeth.map((tooth) => (
        <line
          x1={tooth.x1}
          y1={tooth.y1}
          x2={tooth.x2}
          y2={tooth.y2}
          stroke="currentColor"
          stroke-width="2.6"
        />
      ))}
      <circle cx="12" cy="12" r={RIM} fill="none" stroke="currentColor" stroke-width="1.5" />
      <circle cx="12" cy="12" r="2.3" fill="none" stroke="currentColor" stroke-width="1.5" />
    </svg>
  );
}
