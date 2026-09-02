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
  view: "calendar" | "day" | "settings" | "chart";
  onBack: () => void;
  onSettings: () => void;
  /** Opens the month and year picker. Only the calendar has one to open. */
  onJump: () => void;
  /** Whether the picker is already open, so the title can say so. */
  jumping: boolean;
  /** Which division the chart is, so the header's mark matches the menu bar item
   *  that opened it. 1 is the rashi chart; several can be in the row at once. */
  division: number;
  /** Whether the app is waiting for a location before it will show anything.
   *
   * The header then names the app and offers nothing: there is no month to
   * title, and the settings gear would be a way round the one thing being
   * insisted on. */
  gated: boolean;
}

export function Header(props: Props): JSX.Element {
  const isCalendar = () => props.view === "calendar" && !props.gated;

  // In a lunar month the day view supplies its own title - the tithi, which is
  // what the day is called there - and the date drops to the line below it. The
  // western date is the name only where it is the calendar in force.
  const label = () =>
    props.gated
      ? "Chandra"
      : props.view === "day" && props.selected && !props.title
        ? formatDateHeading(props.selected)
        : props.title;

  const full = () =>
    isCalendar() ? `${props.subjectName} · ${props.title}` : label();

  let labelElement: HTMLElement | undefined;
  const [level, setLevel] = createSignal(0);

  // Ladder. The subject name is the last thing dropped, not the first: it is
  // what tells a Mangala calendar from a Chandra one, and losing it leaves two
  // panels that read identically. The year goes first, then the name.
  const withoutYear = () => props.title.replace(/\s+\d{1,4}$/, "");
  // The chart's title names the division *and* what is rising, which is more
  // than the line always holds - `Chaturvimsamsa · Vrishchika Lagna` is
  // thirty-three characters in a two-hundred-point slot. It drops `Lagna` first,
  // then the division, because the rising sign is the reading and the division is
  // also on the mark beside it.
  const chartRungs = () => {
    const parts = label().split(" · ");
    if (parts.length < 2) return [label()];
    return [label(), label().replace(" Lagna", ""), parts[1]!];
  };

  const candidates = () =>
    isCalendar()
      ? [
          full(),
          `${props.subjectName} · ${withoutYear()}`,
          props.title,
          withoutYear(),
        ]
      : props.view === "chart"
        ? chartRungs()
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
    if (
      element.scrollWidth > element.clientWidth &&
      level() < candidates().length - 1
    ) {
      setLevel(level() + 1);
    }
  });

  return (
    <header class="header">
      {/* The leading slot is empty rather than holding a control that would do
          nothing, in two cases.

          While the gate is up there is nothing to go back to and nowhere to go.

          On the chart there is no hierarchy to climb: it is reached from its own
          status item and is a peer of the calendar, not a step inside it. A
          chevron there would also leave the back end still believing the chart
          was showing, so its own item would hide the panel rather than return
          to it. */}
      <Show
        when={isCalendar()}
        fallback={
          <Show
            when={!props.gated && props.view !== "chart"}
            fallback={
              <Show
                when={props.view === "chart"}
                fallback={<span class="header__icon" />}
              >
                {/* The chart's own mark, the same one its status item carries,
                    so the header reads like every other view's: a glyph, a
                    title, a gear. An empty slot left the title hanging where
                    every other view has something. */}
                <span class="header__glyph">
                  <ChartMark division={props.division} />
                </span>
              </Show>
            }
          >
            <button
              class="header__icon"
              onClick={props.onBack}
              aria-label="Back"
            >
              <Chevron />
            </button>
          </Show>
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
      {/* A button in the calendar and nothing but text elsewhere. The month
          name is the one thing on screen that already names where the strip is,
          so it is where a reader looks to change it - and giving the picker its
          own control would have cost a slot the header does not have.

          Two tags rather than one with a `disabled` attribute: a disabled
          button is announced as dimmed, and a settings section title is not a
          control that has been switched off. */}
      <Show
        when={isCalendar()}
        fallback={
          <div
            class="header__label"
            ref={(element) => (labelElement = element)}
            aria-label={full()}
          >
            <Label adhika={props.adhika} text={fitted()} />
          </div>
        }
      >
        <button
          type="button"
          class="header__label header__label--button"
          classList={{ "is-open": props.jumping }}
          ref={(element) => (labelElement = element)}
          aria-label={`${full()}. Jump to a month`}
          aria-expanded={props.jumping}
          onClick={props.onJump}
        >
          <Label adhika={props.adhika} text={fitted()} />
        </button>
      </Show>

      <Show when={props.view !== "settings" && !props.gated}>
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

/** The fitted label, with `Adhika` set apart where the month is intercalary.
 *
 * Split from the fitted string rather than passed separately, so the measuring
 * ladder above still sees one label. */
function Label(props: { adhika: boolean; text: string }): JSX.Element {
  return (
    <Show
      when={props.adhika ? qualifier(props.text) : null}
      fallback={props.text}
    >
      {(split) => (
        <>
          {split().before}
          <span class="header__qualifier">Adhika </span>
          {split().after}
        </>
      )}
    </Show>
  );
}

function qualifier(label: string): { before: string; after: string } | null {
  const at = label.indexOf(QUALIFIER);
  if (at < 0) return null;
  return {
    before: label.slice(0, at),
    after: label.slice(at + QUALIFIER.length),
  };
}

/** The Lagna Kundali's mark: the North Indian construction reduced to what
 *  survives at this size - the outer square, the midpoint diamond filled solid,
 *  and the part of each diagonal that crosses a corner triangle.
 *
 *  The same shape as the status item, and deliberately not the same weight.
 *  The tray mark was tuned to sit evenly in a 22pt menu bar slot beside the
 *  system's own glyphs, where it measured 45 per cent of its slot in ink against
 *  their 12 to 18 and had to come down to 0.62 of a graha's stroke. This one
 *  sits at 16px beside body text in the panel, where that stroke would be under
 *  three quarters of a pixel and read as a smudge.
 *
 *  What is held in common is the construction and the ratios that make it
 *  legible: the diamond held clear of the square rather than touching it, and
 *  the stubs run half way from each corner to the centre, which is exactly where
 *  the diamond's edge crosses the diagonal. Drawn twice in two languages, so the
 *  numbers are written down rather than left to be re-derived from the other. */
function ChartMark(props: { division: number }): JSX.Element {
  return (
    <svg width="16" height="16" viewBox="0 0 16 16" aria-hidden="true">
      <Show
        when={props.division > 1}
        fallback={
          <>
            <rect
              x="1.92"
              y="1.92"
              width="12.16"
              height="12.16"
              fill="none"
              stroke="currentColor"
              stroke-width="1.2"
            />
            <path d="M 8 3.3 L 12.7 8 L 8 12.7 L 3.3 8 Z" fill="currentColor" />
            <path
              d="M 1.92 1.92 L 4.96 4.96 M 14.08 1.92 L 11.04 4.96 M 14.08 14.08 L 11.04 11.04 M 1.92 14.08 L 4.96 11.04"
              fill="none"
              stroke="currentColor"
              stroke-linecap="round"
              stroke-width="0.96"
            />
          </>
        }
      >
        {/* The numeral alone, as the status item draws it. The frame was here
            too and had to go for the same reason it went there: with it, two
            digits are too small to read. D1 keeps it, and D1 is always in the
            row, so one mark in the menu bar still says what the numbers beside
            it are counting.

            Real type rather than the tray's seven segments, because this is a
            web view with a font in it - and a numeral set in the panel's own
            face is more legible at 16px than segments would be. */}
        <text
          class="header__division"
          x="8"
          y="12.4"
          text-anchor="middle"
          fill="currentColor"
        >
          {props.division}
        </text>
      </Show>
    </svg>
  );
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
      <circle
        cx="12"
        cy="12"
        r={RIM}
        fill="none"
        stroke="currentColor"
        stroke-width="1.6"
      />
      <circle
        cx="12"
        cy="12"
        r="2.3"
        fill="none"
        stroke="currentColor"
        stroke-width="1.6"
      />
    </svg>
  );
}
