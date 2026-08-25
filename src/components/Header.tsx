/**
 * Panel header.
 *
 * Carries the subject and the month in the calendar, and the way back
 * everywhere else. There are no month arrows: months change by scrolling or
 * dragging the grid, which works the same for a lunar month, where stepping is
 * from one syzygy to the next rather than through a numbered sequence.
 */

import type { JSX } from "solid-js";
import { createEffect, createSignal, on, Show } from "solid-js";

import type { DateKey, GrahaInfo, GrahaKey, Snapshot } from "../ipc/types";
import { formatDateHeading } from "../lib/format";
import { GrahaGlyph } from "./GrahaGlyph";
import { PhaseGlyph } from "./PhaseGlyph";

interface Props {
  subject: GrahaKey;
  subjectName: string;
  info: GrahaInfo | undefined;
  snapshot: Snapshot | undefined;
  southern: boolean;
  /** Month label in the calendar, section name in settings. */
  title: string;
  /** Whether the month in view is intercalary, so `Adhika` can be set apart. */
  adhika: boolean;
  selected: DateKey | null;
  view: "calendar" | "day" | "settings";
  onBack: () => void;
  onSettings: () => void;
}

export function Header(props: Props): JSX.Element {
  const isCalendar = () => props.view === "calendar";

  // In a lunar month the day view supplies its own title - the tithi, which is
  // what the day is called there - and the date drops to the line below it. The
  // western date is the name only where it is the calendar in force.
  const label = () =>
    props.view === "day" && props.selected && !props.title
      ? formatDateHeading(props.selected)
      : props.title;

  const full = () =>
    isCalendar() ? `${props.subjectName} · ${props.title}` : label();

  let labelElement: HTMLDivElement | undefined;
  const [level, setLevel] = createSignal(0);

  // Ladder. The subject name is the last thing dropped, not the first: it is
  // what tells a Mangala calendar from a Chandra one, and losing it leaves two
  // panels that read identically. The year goes first, then the name.
  const withoutYear = () => props.title.replace(/\s+\d{1,4}$/, "");
  const candidates = () =>
    isCalendar()
      ? [
          full(),
          `${props.subjectName} · ${withoutYear()}`,
          props.title,
          withoutYear(),
        ]
      : [label()];
  const fitted = () => candidates()[level()] ?? full();

  createEffect(
    on(
      () => [props.subjectName, props.title, props.view],
      () => setLevel(0),
    ),
  );

  // Measured from real layout. Assigning a computed font shorthand to a canvas
  // context silently fails for `-apple-system`, so the canvas would report every
  // candidate as fitting and the label would clip mid-character.
  createEffect(() => {
    const current = fitted();
    void current;
    const element = labelElement;
    if (!element) return;
    if (element.scrollWidth > element.clientWidth && level() < candidates().length - 1) {
      setLevel(level() + 1);
    }
  });

  return (
    <header class="header">
      <Show
        when={isCalendar()}
        fallback={
          <button class="header__icon" onClick={props.onBack} aria-label="Back">
            <Chevron />
          </button>
        }
      >
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
      </Show>

      {/* `Adhika` is a qualifier on the month name, not part of it. Setting it
          apart is what stops `Adhika Shravana` from reading as a thirteenth
          month name of its own. Split from the fitted string rather than passed
          separately, so the measuring ladder above still sees one label. */}
      <div class="header__label" ref={labelElement} aria-label={full()}>
        <Show when={props.adhika ? qualifier(fitted()) : null} fallback={fitted()}>
          {(split) => (
            <>
              {split().before}
              <span class="header__qualifier">Adhika </span>
              {split().after}
            </>
          )}
        </Show>
      </div>

      <Show when={props.view !== "settings"}>
        <button
          class="header__icon"
          onClick={props.onSettings}
          aria-label="Settings"
        >
          <Gear />
        </button>
      </Show>
    </header>
  );
}

/**
 * Splits a label around its `Adhika` prefix.
 *
 * Only called for a month that carries the flag. Searching the label for the
 * word was re-deriving from a string what the payload already stated, and would
 * have set the qualifier apart on any month whose name happened to contain it.
 */
const QUALIFIER = "Adhika ";

function qualifier(label: string): { before: string; after: string } | null {
  const at = label.indexOf(QUALIFIER);
  if (at < 0) return null;
  return { before: label.slice(0, at), after: label.slice(at + QUALIFIER.length) };
}

function Chevron(): JSX.Element {
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" aria-hidden="true">
      <path
        d="M 14.5 5 L 8.5 12 L 14.5 19"
        fill="none"
        stroke="currentColor"
        stroke-width="1.7"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
  );
}

function Gear(): JSX.Element {
  // Teeth start at the rim and are short and blunt. Long thin spokes standing
  // clear of the circle read as a sun rather than a gear.
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
    <svg width="20" height="20" viewBox="0 0 24 24" aria-hidden="true">
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
      <circle cx="12" cy="12" r={RIM} fill="none" stroke="currentColor" stroke-width="1.6" />
      <circle cx="12" cy="12" r="2.3" fill="none" stroke="currentColor" stroke-width="1.6" />
    </svg>
  );
}
