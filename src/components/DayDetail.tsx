/**
 * The expanded day detail.
 *
 * One shape for all nine subjects and both calendars: a stack of fields, each a
 * small label, the value, and the hour it gives way. What differs between
 * subjects is which fields exist, never how they are drawn.
 *
 * The four numbers this view used to lead with - illuminated percentage,
 * distance from the Sun, speed, longitude - are gone. They were the only reason
 * the Moon's view and a graha's had different shapes, and none of them was what
 * anyone opened the day to read.
 *
 * Rise and set belong to the subject. Surya's day names sunrise and sunset,
 * Chandra's names moonrise and moonset, and a graha's names its own. There is no
 * separate sunrise row on every subject's day: sunrise is Surya's rise.
 *
 * Degraded readings - no moonrise, a boundary that did not resolve, a Moshier
 * position - are stated as facts, with no warning colour and no icon: they are
 * normal, not faults.
 */

import type { JSX } from "solid-js";
import { createSignal, Index, Show } from "solid-js";

import type {
  DayPanchanga,
  DayDetail as Detail,
  Dignity,
  GrahaDay,
  GrahaKey,
  KaranaSpan,
  MoonDay,
  NakshatraSpan,
  RashiSpan,
  Standing,
  TithiSpan,
  TransitEvent,
  YogaSpan,
} from "../ipc/types";
import type { FormatContext } from "../lib/format";
import {
  formatShortDate,
  formatTime,
  formatUntil,
  formatWeekday,
  phaseLabel,
  zoneDiffersFromMachine,
} from "../lib/format";
import { describeEvent } from "./MonthGrid";

/** Display names, for a row that names whose rise it is. */
const GRAHA_NAMES: Record<GrahaKey, string> = {
  surya: "Surya",
  chandra: "Chandra",
  mangala: "Mangala",
  budha: "Budha",
  guru: "Guru",
  shukra: "Shukra",
  shani: "Shani",
  rahu: "Rahu",
  ketu: "Ketu",
};

interface Props {
  detail: Detail | undefined;
  events: TransitEvent[];
  context: FormatContext;
  isToday: boolean;
  /** Whether the calendar in force names months by the Moon.
   *
   * The payload no longer says: a day carries its panchanga in both calendars.
   * Two things still turn on it - whether the header is showing the tithi as
   * the title, and therefore whether this view has to print the civil date and
   * the Moon's phase itself. */
  lunar: boolean;
  error: { code: string; message: string } | undefined;
  /** Steps the day being read, in days. Optional because the visual harness
   *  draws this pane without a calendar behind it to step through. */
  onStep?: (delta: number) => void;
  /** Whether what is drawn is the previous day, still on screen while the one
   *  asked for is fetched. */
  stale?: boolean;
}

/** Which pane is showing. */
type Pane = "day" | "position";

/* The two states the day carries on its surface as well as in its fields.
   Each is one predicate used in both places, so the drawing and the words
   cannot disagree - the guarantee `the_combustion_mark_and_the_day_it_opens_
   agree` gives the grid, held here by construction instead of by a test. */

/** Combust, and knowably so: `orb` is absent for the Sun and the nodes. */
function isCombust(detail: Detail | undefined): boolean {
  return (
    detail?.combustion.orb !== null && (detail?.combustion.combust ?? false)
  );
}

/** Retrograde. No subject is excluded.
 *
 * Rahu and Ketu were, on the grounds that they are retrograde on roughly 95% of
 * days and a mark that is true almost always is the subject's identity rather
 * than its state. Both halves of that were wrong.
 *
 * Measured over twenty years: a *mean* node is retrograde on 100% of days, and
 * a *true* node on 74.1%, turning
 * direct about twenty-five times a year for a little under four days at a time.
 * The true node oscillates about the mean with a fortnightly term, and that
 * oscillation outruns the mean retrograde rate for part of every half draconic
 * month. So the state does change, and often.
 *
 * The exclusion was also a disagreement with the grid, which never had one: a
 * cell drew the bracket for Rahu and Ketu from the same flag while the day it
 * opened drew nothing. Under a mean node the rail is permanent, and that is
 * truthful rather than noisy - it is the model the user chose. */
function hasRetrogradeRail(detail: Detail | undefined): boolean {
  if (detail?.kind !== "graha") return false;
  return (detail as GrahaDay).retrograde;
}

export function DayDetail(props: Props): JSX.Element {
  return (
    <div
      class="detail"
      classList={{ "is-stale": props.stale }}
      role="region"
      aria-live="polite"
      aria-busy={props.stale ? "true" : undefined}
      aria-label="Day detail"
    >
      {/* The two states the surface carries, drawn behind the fields rather than
          added to them. Neither costs a vertical pixel, which is what lets the
          day keep gaining fields. Both are said in words as well - a Combust
          field and a Motion field - so nothing here is a sole carrier. */}
      <Show when={!props.error && isCombust(props.detail)}>
        <span class="detail__glare" aria-hidden="true" />
      </Show>
      <Show when={!props.error && hasRetrogradeRail(props.detail)}>
        <span class="detail__rail" aria-hidden="true" />
      </Show>

      <Show when={props.error} fallback={<Body {...props} />}>
        {(error) => (
          <ErrorBlock code={error().code} message={error().message} />
        )}
      </Show>
    </div>
  );
}

function Body(props: Props): JSX.Element {
  /**
   * Which pane is showing, per panel rather than per day.
   *
   * Opening a second day keeps the pane you were in: a pane that reset on every
   * open would make comparing one field across a run of days a two-click
   * operation, which is most of what a calendar is for.
   *
   * Position opens first, for every subject. It is the pane whose contents
   * change with the subject, so it is the one the calendar you are in is about;
   * the Day pane is identical on all nine panels, which is what makes it the one
   * you go to deliberately.
   */
  const [pane, setPane] = createSignal<Pane>("position");

  return (
    <Show when={props.detail}>
      {(detail) => (
        <>
          {/* The header already carries the date, so this names the day rather
              than repeating it: the weekday, and the vara, which is what the day
              is actually called. */}
          {/* The day steps from here. Reading one day after another is most
              of what this view is for, and every step used to mean going back
              to the grid and picking again - two moves and a change of view to
              see tomorrow. The arrows flank the line that names the day, so
              what they move is the thing beside them. Left and right, not up
              and down: a day's neighbours are before and after it. */}
          <div class="detail__step">
            <Show when={props.onStep} fallback={<span class="detail__nudge" />}>
              {(step) => (
                <button
                  class="detail__nudge"
                  onClick={() => step()(-1)}
                  aria-label="Previous day"
                >
                  <Nudge back />
                </button>
              )}
            </Show>
            <div class="detail__date">
            <Show when={props.isToday}>
              <span class="detail__today">TODAY</span>
              <span> · </span>
            </Show>
            {formatWeekday(detail().date)}
            {/* The civil date, where the header is carrying the tithi instead.
                A lunar day still has to be findable in the world the user
                lives in. */}
            <Show when={props.lunar}>
              <span> · {formatShortDate(detail().date)}</span>
            </Show>
            <span> · {detail().panchanga.vara_name}</span>
            </div>
            <Show when={props.onStep} fallback={<span class="detail__nudge" />}>
              {(step) => (
                <button
                  class="detail__nudge"
                  onClick={() => step()(1)}
                  aria-label="Next day"
                >
                  <Nudge />
                </button>
              )}
            </Show>
          </div>

          <div class="detail__panes" role="tablist" aria-label="Day detail">
            <PaneTab pane="day" current={pane()} onPick={setPane}>
              Day
            </PaneTab>
            <PaneTab pane="position" current={pane()} onPick={setPane}>
              Position
            </PaneTab>
          </div>

          <div
            class="detail__scroll"
            role="tabpanel"
            aria-label={pane() === "day" ? "Day" : "Position"}
          >
            <Show
              when={pane() === "day"}
              fallback={
                <PositionPane
                  detail={detail()}
                  events={props.events}
                  context={props.context}
                  lunar={props.lunar}
                />
              }
            >
              <DayPane detail={detail()} context={props.context} />
            </Show>

            {/* The rule a day is named by, when it is not the usual one.
                A panchanga names the day after the tithi prevailing at sunrise.
                Inside a polar day or night the Sun does not rise, so local noon
                is used instead - it keeps the reference inside the day and keeps
                the calendar usable where the traditional rule has nothing to
                point at.

                The payload has carried this distinction since the rule was
                written and nothing read it, so at Longyearbyen in December the
                day was named by a different rule than the one every other day
                uses and the surface said nothing. That is the same failure the
                precision note exists to prevent, one rule down. */}
            <Show when={detail().panchanga.reference === "local_noon"}>
              <p class="detail__provenance">
                The Sun does not rise today. This day is named for local noon,
                not sunrise.
              </p>
            </Show>

            <Show when={detail().source === "moshier"}>
              {/* Named for what it means, not for the theory that produced it:
                  "Moshier ephemeris" is the name of a piece of arithmetic and
                  tells a reader nothing. The magnitude is given so it reads as a
                  fact rather than a warning - a second is nothing against times
                  printed to the minute, and a reader who is not told that has to
                  assume the worst. */}
              <p class="detail__provenance">
                Outside 1800–2399. Times here are approximate, by about a
                second.
              </p>
            </Show>

            {/* Every time in the app is in the observer's zone, which is the
                right answer - a sunrise is a fact about a place. It is also
                silently wrong-looking when that is not this Mac's zone, and
                nothing said so. Printed only when the two differ: for almost
                everyone they are the same, and a standing note would be noise
                on every day. */}
            <Show when={zoneDiffersFromMachine(props.context)}>
              <p class="detail__provenance">
                Times are {props.context.timeZone.replace(/_/g, " ")}, not this
                Mac&rsquo;s clock.
              </p>
            </Show>
          </div>
        </>
      )}
    </Show>
  );
}

/** One of the two pane tabs. */
function PaneTab(props: {
  pane: Pane;
  current: Pane;
  onPick: (pane: Pane) => void;
  children: JSX.Element;
}): JSX.Element {
  const selected = () => props.current === props.pane;
  return (
    <button
      type="button"
      class="detail__pane"
      classList={{ "is-current": selected() }}
      role="tab"
      // The state is announced, not left to the ground colour. Which pane is
      // showing is the one thing on this control a reader has to know.
      aria-selected={selected()}
      onClick={() => props.onPick(props.pane)}
    >
      {props.children}
    </button>
  );
}

/** A field: what it is, what it says, and when that stops being true. */
function Field(props: {
  label: string;
  value: JSX.Element;
  when?: string;
  then?: string;
  spoken?: string;
}): JSX.Element {
  return (
    <div
      class="field"
      role="group"
      aria-label={`${props.label}, ${props.spoken ?? ""}`}
    >
      <div class="field__key">{props.label}</div>
      <div class="field__line">
        <span class="field__value">{props.value}</span>
        <Show when={props.when}>
          {(when) => <span class="field__time">{when()}</span>}
        </Show>
      </div>
      <Show when={props.then}>
        {(next) => <div class="field__next">then {next()}</div>}
      </Show>
    </div>
  );
}

/**
 * What the civil day is, regardless of what is being plotted on it.
 *
 * Identical on all nine panels, and that is the split: a tithi does not depend
 * on which graha is being read against it. Nothing here names the subject.
 *
 * The Moon's nakshatra is the panchanga's fifth limb and is deliberately not
 * here. The payload carries the *subject's* nakshatra, which on a Mangala day is
 * Mangala's - so a Nakshatra row in this pane would be true only when the
 * subject happened to be the Moon. It sits in Position, where it is the
 * subject's and is labelled as such.
 */
function DayPane(props: {
  detail: Detail;
  context: FormatContext;
}): JSX.Element {
  const day = () => props.detail.panchanga;

  return (
    <>
      <Spans label="Tithi" spans={tithiRows(day(), props.context)} />

      <Show when={day().yogas.length > 0}>
        <Spans label="Yoga" spans={yogaRows(day().yogas, props.context)} />
      </Show>

      <Show when={day().karanas.length > 0}>
        <Spans
          label="Karana"
          spans={karanaRows(day().karanas, props.context)}
        />
      </Show>

      {/* Sunrise and sunset on one line, the two ends of the same fact. This is
          the day's own light, not a subject's rise: Surya's rise is in Position
          on Surya's panel, and says the same thing about a different thing. */}
      <Field
        label="Daylight"
        value={
          day().sunrise
            ? formatTime(day().sunrise!, props.context)
            : "the Sun does not rise"
        }
        when={
          day().sunset
            ? `sets ${formatTime(day().sunset!, props.context)}`
            : undefined
        }
        spoken={`${
          day().sunrise
            ? `sunrise ${formatTime(day().sunrise!, props.context)}`
            : "the Sun does not rise"
        }${day().sunset ? `, sets ${formatTime(day().sunset!, props.context)}` : ""}`}
      />

      {/* Every window the day is divided into, in the order they begin. Empty
          where the Sun does not rise and set: a window defined as a fraction of
          daylight has no meaning on a day with none. */}
      <Show when={day().muhurtas.length > 0}>
        <div class="field">
          <div class="field__key">Muhurtas</div>
          <Index each={day().muhurtas}>
            {(window) => (
              <div
                class="field__line"
                role="group"
                aria-label={`${window().name}, ${
                  window().inauspicious ? "avoid" : "auspicious"
                }, ${formatTime(window().start, props.context)} to ${formatTime(
                  window().end,
                  props.context,
                )}`}
              >
                <span class="field__value">{window().name}</span>
                <span class="field__time">
                  {formatTime(window().start, props.context)}–
                  {formatTime(window().end, props.context)}
                </span>
              </div>
            )}
          </Index>
        </div>
      </Show>
    </>
  );
}

/**
 * Where the subject stands, and what that puts it in relation to.
 *
 * Nothing here is the same for two subjects, which is the other half of the
 * split.
 */
function PositionPane(props: {
  detail: Detail;
  events: TransitEvent[];
  context: FormatContext;
  lunar: boolean;
}): JSX.Element {
  const moon = () =>
    props.detail.kind === "moon" ? (props.detail as MoonDay) : null;
  const graha = () =>
    props.detail.kind === "graha" ? (props.detail as GrahaDay) : null;
  const standing = (): Standing => props.detail.standing;

  const isNode = () => graha()?.graha === "rahu" || graha()?.graha === "ketu";
  const rise = () => (moon() ? moon()!.moonrise : graha()!.rise);
  const set = () => (moon() ? moon()!.moonset : graha()!.set);

  /**
   * The subject's own rise, named for the subject.
   *
   * Surya's day names sunrise and Chandra's moonrise because those are the words
   * for them; every other graha is named outright rather than given a bare
   * "Rise", so the row says whose rise it is without the header having to.
   */
  const riseLabel = () => {
    if (moon()) return "Moonrise";
    const body = graha()!.graha;
    if (body === "surya") return "Sunrise";
    return `${GRAHA_NAMES[body]} rise`;
  };

  return (
    <>
      {/* The phase names what a Gregorian day shows of the Moon. In a lunar
          month the tithi says the same thing more precisely and is the header's
          title, so it does not appear twice. */}
      <Show when={moon() && !props.lunar}>
        <Field
          label="Phase"
          value={phaseLabel(moon()!.phase)}
          spoken={phaseLabel(moon()!.phase)}
        />
      </Show>

      <Spans
        label="Rashi"
        spans={rashiRows(props.detail.rashis, props.context)}
      />

      <Spans
        label="Nakshatra"
        spans={nakshatraRows(props.detail.nakshatras, props.context)}
      />

      {/* Whose ground it is standing on. A separate field from the nakshatra
          above rather than a suffix on it, because the lord belongs to the
          nakshatra prevailing at the reference instant and the row above names
          its successor too - a suffix would have read as qualifying both. */}
      <Field
        label="Nakshatra lord"
        value={GRAHA_NAMES[standing().nakshatra_lord]}
        spoken={`in the nakshatra of ${GRAHA_NAMES[standing().nakshatra_lord]}`}
      />

      {/* Named in words as well as by the rail in the margin. A minus sign on a
          speed is not a state anyone should have to infer - and the speed itself
          is gone. */}
      <Show when={graha()}>
        {(body) => (
          <Field
            label="Motion"
            value={
              <>
                {body().retrograde ? "Retrograde" : "Direct"}
                <Show when={body().retrograde}>
                  <span class="chip" aria-hidden="true">
                    (r)
                  </span>
                </Show>
              </>
            }
            spoken={body().retrograde ? "retrograde" : "direct"}
          />
        )}
      </Show>

      <Show when={standing().dignity}>
        {(dignity) => (
          <Field
            label="Dignity"
            value={DIGNITY_LABELS[dignity()]}
            spoken={DIGNITY_LABELS[dignity()]}
          />
        )}
      </Show>

      {/* Cast and received on one field, because they are the same relation read
          from the two ends and a reader looking for one is looking for both. */}
      <Show
        when={standing().aspects.length + standing().aspected_by.length > 0}
      >
        <div class="field">
          <div class="field__key">Drishti</div>
          <Show when={standing().aspects.length > 0}>
            <div
              class="field__line"
              role="group"
              aria-label={`aspects ${names(standing().aspects)}`}
            >
              <span class="field__value">
                aspects {names(standing().aspects)}
              </span>
            </div>
          </Show>
          <Show when={standing().aspected_by.length > 0}>
            <div
              class="field__line"
              role="group"
              aria-label={`aspected by ${names(standing().aspected_by)}`}
            >
              <span class="field__value">
                aspected by {names(standing().aspected_by)}
              </span>
            </div>
          </Show>
        </div>
      </Show>

      {/* No winner. Which graha wins a graha yuddha is decided differently by
          different authorities, so the pairing and the separation are what is
          reported - the observation rather than a reading of it. */}
      <Show when={standing().war}>
        {(war) => (
          <Field
            label="Planetary war"
            value={`with ${GRAHA_NAMES[war().with]}`}
            when={`${war().separation.toFixed(2)}°`}
            spoken={`at war with ${GRAHA_NAMES[war().with]}, ${war().separation.toFixed(
              2,
            )} degrees apart`}
          />
        )}
      </Show>

      {/* Rise and set on one line: two ends of the same fact, and a body that
          does not set says so rather than leaving a blank.

          Absent for the nodes. They are points on the ecliptic and do cross the
          horizon; the app declines to model it, and "does not rise" would report
          that decision as an observation - the same sentence a circumpolar Moon
          gets for a completely different reason. */}
      <Show when={!isNode()}>
        <Field
          label={riseLabel()}
          value={rise() ? formatTime(rise()!, props.context) : "does not rise"}
          when={
            set() ? `sets ${formatTime(set()!, props.context)}` : "does not set"
          }
          spoken={`${rise() ? formatTime(rise()!, props.context) : "does not rise"}, ${
            set() ? `sets ${formatTime(set()!, props.context)}` : "does not set"
          }`}
        />
      </Show>

      {/* Combustion is drawn on the surface, so this is the words that carry it
          for anyone the drawing does not reach (DESIGN 11.3). The separation in
          degrees is not here: the orb is the fact, the reading behind it is not
          what the day was opened for. */}
      <Show when={isCombust(props.detail)}>
        <Field
          label="Combust"
          value={`inside the ${props.detail.combustion.orb}° orb`}
          spoken={`combust, inside the ${props.detail.combustion.orb} degree orb`}
        />
      </Show>

      <Show when={props.events.length > 0}>
        <div class="field">
          {/* Not "Today": these are the selected day's, and the selected day is
              usually not today. The header above already names the date. */}
          <div class="field__key">Events</div>
          <Index each={props.events}>
            {(event) => (
              <div class="field__line">
                <span class="field__value">{describeEvent(event())}</span>
                <span class="field__time">
                  {formatTime(event().at, props.context)}
                </span>
              </div>
            )}
          </Index>
        </div>
      </Show>
    </>
  );
}

const DIGNITY_LABELS: Record<Dignity, string> = {
  exalted: "Exalted",
  debilitated: "Debilitated",
  own_sign: "Own sign",
};

/** `Guru and Shani`, `Budha, Guru and Shani`. */
function names(grahas: GrahaKey[]): string {
  const words = grahas.map((graha) => GRAHA_NAMES[graha]);
  if (words.length <= 1) return words.join("");
  return `${words.slice(0, -1).join(", ")} and ${words[words.length - 1]}`;
}

interface SpanRow {
  value: string;
  until: string;
  prevailing: boolean;
}

/**
 * One span field.
 *
 * The prevailing span is the value, and what follows it *in time* is named on
 * the `then` line. The spans arrive in the order they occupy the day, so the
 * successor is the next index - not merely the next one that is not prevailing,
 * which on any day whose reference instant falls in the second span names the
 * one already gone. The Moon moving Dhanu to Makara read "Makara, then Dhanu".
 */
function Spans(props: { label: string; spans: SpanRow[] }): JSX.Element {
  const index = () => {
    const at = props.spans.findIndex((span) => span.prevailing);
    return at < 0 ? 0 : at;
  };
  const current = () => props.spans[index()];
  const next = () => props.spans[index() + 1];

  return (
    <Show when={current()}>
      {(span) => (
        <Field
          label={props.label}
          value={span().value}
          when={span().until}
          then={next()?.value}
          spoken={`${span().value}, until ${span().until}`}
        />
      )}
    </Show>
  );
}

/**
 * A tithi's rows, carrying why a number was skipped or repeated.
 *
 * `sunrises` is `null` when the question could not be put - an unresolved
 * boundary, or a latitude where the Sun rose on none of the three days - and
 * says nothing. Reading it as zero captioned an ordinary tithi as one no day is
 * named after.
 */
function tithiRows(panchanga: DayPanchanga, context: FormatContext): SpanRow[] {
  return panchanga.tithis.map((span: TithiSpan) => {
    // A kshaya is a property of the tithi: it is one no day is named after, and
    // saying so explains a number missing from the grid. "Two sunrises" was the
    // same fact from the other end and did not survive the trip - it is a
    // statement about the tithi read beside a heading that names one civil day,
    // where it reads as a claim that the day had two dawns. The repeat is
    // already visible in the grid, where the same numeral appears twice.
    const note = span.sunrises === 0 ? " · kshaya" : "";
    return {
      value: `${span.paksha === "shukla" ? "Shukla" : "Krishna"} ${span.name}${note}`,
      until: span.exit ? formatUntil(span.exit, context) : "time unavailable",
      prevailing: span.prevailing,
    };
  });
}

function yogaRows(spans: YogaSpan[], context: FormatContext): SpanRow[] {
  return spans.map((span) => ({
    value: span.name,
    until: span.exit ? formatUntil(span.exit, context) : "time unavailable",
    prevailing: span.prevailing,
  }));
}

/**
 * A karana's rows.
 *
 * The four that occur once a lunar month are marked as such. Without it Shakuni
 * and Bava read as the same kind of thing, and one of them will not be seen
 * again for a month - which is the only reason a reader would look this up.
 */
function karanaRows(spans: KaranaSpan[], context: FormatContext): SpanRow[] {
  return spans.map((span) => ({
    value: span.fixed ? `${span.name} · fixed` : span.name,
    until: span.exit ? formatUntil(span.exit, context) : "time unavailable",
    prevailing: span.prevailing,
  }));
}

function nakshatraRows(
  spans: NakshatraSpan[],
  context: FormatContext,
): SpanRow[] {
  return spans.map((span) => ({
    value: `${span.name} · Pada ${span.pada}`,
    until: formatUntil(span.exit, context),
    prevailing: span.prevailing,
  }));
}

function rashiRows(spans: RashiSpan[], context: FormatContext): SpanRow[] {
  return spans.map((span) => ({
    value: span.name,
    until: formatUntil(span.exit, context),
    prevailing: span.prevailing,
  }));
}

const ERROR_TEXT: Record<string, { headline: string; cause: string }> = {
  // Not "Outside 1800-2399", which is what this said. The back end raises this
  // for a year, month and day that do not name a day - 30 February, month 13 -
  // and there is no out-of-range case at all: outside 1800-2399 the analytic
  // fallback answers and the panel says so. Both sentences were false for the
  // one input that could reach them.
  INVALID_DATE: {
    headline: "Not a date.",
    cause: "That day does not exist in the calendar.",
  },
  NO_CONVERGENCE: {
    headline: "Boundary time unavailable.",
    cause: "This entry could not be resolved.",
  },
  SETTINGS: {
    headline: "Settings could not be saved.",
    cause: "",
  },
  // The back end's catch-all: an ephemeris failure reaches it, and so does a
  // poisoned lock, a failed dispatch to the main thread, and any almanac error
  // with no code of its own. "Ephemeris unavailable" named one of those as the
  // cause of all of them, which is a guess printed as a fact. The message
  // underneath says what actually happened.
  ENGINE: {
    headline: "This could not be computed.",
    cause: "",
  },
  // Not a fault, and the only code here with a remedy the reader can apply.
  // `chakra` and `now` re-read whenever the settings change under them, so
  // nothing half-computed is ever returned; after three attempts they give up
  // rather than spin. It used to arrive as ENGINE with the message "ephemeris:
  // time zone: the configuration kept changing while the chart was being read".
  BUSY: {
    headline: "Interrupted by a settings change.",
    cause: "Nothing is wrong. Open it again.",
  },
};

/**
 * A code the front end does not know.
 *
 * Its own entry rather than borrowing `ENGINE`'s: a code from a newer back end
 * has no known cause here, and reusing another code's headline asserts one.
 */
const UNKNOWN_ERROR = { headline: "Something went wrong.", cause: "" };

export function ErrorBlock(props: {
  code: string;
  message: string;
}): JSX.Element {
  const text = () => ERROR_TEXT[props.code] ?? UNKNOWN_ERROR;
  return (
    <div class="error-block">
      <p class="error-block__headline">{text().headline}</p>
      <p class="error-block__cause">
        {text().cause || truncate(props.message)}
      </p>
    </div>
  );
}

function truncate(message: string): string {
  return message.length > 60 ? `${message.slice(0, 59)}…` : message;
}

/**
 * The chevron on a day-step button.
 *
 * The same shape as the header's back chevron, at the size a control beside a
 * line of micro type can be: the panel has one chevron and this is it, turned
 * around for the forward direction rather than drawn a second time.
 */
function Nudge(props: { back?: boolean }): JSX.Element {
  return (
    <svg
      width="14"
      height="14"
      viewBox="0 0 24 24"
      aria-hidden="true"
      style={props.back ? undefined : { transform: "scaleX(-1)" }}
    >
      <path
        d="M 14.5 5 L 8.5 12 L 14.5 19"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
  );
}
