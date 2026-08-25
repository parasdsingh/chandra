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
import { Index, Show } from "solid-js";

import type {
  DayPanchanga,
  DayDetail as Detail,
  GrahaDay,
  GrahaKey,
  MoonDay,
  NakshatraSpan,
  RashiSpan,
  TithiSpan,
  TransitEvent,
} from "../ipc/types";
import type { FormatContext } from "../lib/format";
import {
  formatShortDate,
  formatTime,
  formatUntil,
  formatWeekday,
  phaseLabel,
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
  error: { code: string; message: string } | undefined;
}

/* The two states the day carries on its surface as well as in its fields.
   Each is one predicate used in both places, so the drawing and the words
   cannot disagree - the guarantee `the_combustion_mark_and_the_day_it_opens_
   agree` gives the grid, held here by construction instead of by a test. */

/** Combust, and knowably so: `orb` is absent for the Sun and the nodes. */
function isCombust(detail: Detail | undefined): boolean {
  return detail?.combustion.orb !== null && (detail?.combustion.combust ?? false);
}

/** Retrograde, excluding the nodes.
 *
 * Rahu and Ketu are retrograde on roughly 95% of days, so a rail on their
 * panels would be their identity rather than a state - two of the nine panels
 * permanently different for no information. The `Motion` field still prints,
 * so nothing is lost in words. */
function hasRetrogradeRail(detail: Detail | undefined): boolean {
  if (detail?.kind !== "graha") return false;
  const graha = detail as GrahaDay;
  return (
    graha.retrograde && graha.graha !== "rahu" && graha.graha !== "ketu"
  );
}

export function DayDetail(props: Props): JSX.Element {
  return (
    <div
      class="detail"
      role="region"
      aria-live="polite"
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
        {(error) => <ErrorBlock code={error().code} message={error().message} />}
      </Show>
    </div>
  );
}

function Body(props: Props): JSX.Element {
  return (
    <Show when={props.detail}>
      {(detail) => (
        <>
          {/* The header already carries the date, so this names the day rather
              than repeating it: the weekday, and in a lunar month the vara,
              which is what the day is actually called. */}
          <div class="detail__date">
            <Show when={props.isToday}>
              <span class="detail__today">TODAY</span>
              <span> · </span>
            </Show>
            {formatWeekday(detail().date)}
            {/* The civil date, where the header is carrying the tithi instead.
                A lunar day still has to be findable in the world the user
                lives in. */}
            <Show when={detail().panchanga}>
              {(panchanga) => (
                <>
                  <span> · {formatShortDate(detail().date)}</span>
                  <span> · {panchanga().vara_name}</span>
                </>
              )}
            </Show>
          </div>

          <Subject
            detail={detail()}
            events={props.events}
            context={props.context}
          />

          <Show when={detail().source === "moshier"}>
            {/* Named for what it means, not for the theory that produced it:
                "Moshier ephemeris" is the name of a piece of arithmetic and
                tells a reader nothing. The magnitude is given so it reads as a
                fact rather than a warning - a second is nothing against times
                printed to the minute, and a reader who is not told that has to
                assume the worst. */}
            <p class="detail__provenance">
              Outside 1800–2399. Times here are approximate, by about a second.
            </p>
          </Show>
        </>
      )}
    </Show>
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
 * Every field of a day, for whichever subject it belongs to.
 *
 * The spans - tithi, nakshatra, rashi - are read the same way: the one in force
 * at the reference instant is the value, the hour it ends is the time, and the
 * one that follows is named underneath. A day holds at most two of each, so
 * naming the successor costs one line and saves opening tomorrow.
 */
function Subject(props: {
  detail: Detail;
  events: TransitEvent[];
  context: FormatContext;
}): JSX.Element {
  const moon = () => (props.detail.kind === "moon" ? (props.detail as MoonDay) : null);
  const graha = () =>
    props.detail.kind === "graha" ? (props.detail as GrahaDay) : null;
  const panchanga = () => props.detail.panchanga;

  const isNode = () =>
    graha()?.graha === "rahu" || graha()?.graha === "ketu";
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
          month the tithi says the same thing more precisely, so it does not
          appear twice. */}
      <Show when={moon() && !panchanga()}>
        <Field label="Phase" value={phaseLabel(moon()!.phase)} spoken={phaseLabel(moon()!.phase)} />
      </Show>

      <Show when={panchanga()}>
        {(day) => (
          <Spans label="Tithi" spans={tithiRows(day(), props.context)} />
        )}
      </Show>

      <Spans
        label="Nakshatra"
        spans={nakshatraRows(props.detail.nakshatras, props.context)}
      />

      <Spans label="Rashi" spans={rashiRows(props.detail.rashis, props.context)} />

      {/* Named in words as well as by the mark on the grid. A minus sign on a
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
                  <span class="chip" aria-hidden="true">℞</span>
                </Show>
              </>
            }
            spoken={body().retrograde ? "retrograde" : "direct"}
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
        when={set() ? `sets ${formatTime(set()!, props.context)}` : "does not set"}
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

function nakshatraRows(spans: NakshatraSpan[], context: FormatContext): SpanRow[] {
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
  DATE_OUT_OF_RANGE: {
    headline: "Outside 1800–2399.",
    cause: "Chandra has no ephemeris data for this date.",
  },
  NO_CONVERGENCE: {
    headline: "Boundary time unavailable.",
    cause: "This entry could not be resolved.",
  },
  SETTINGS: {
    headline: "Settings could not be saved.",
    cause: "",
  },
  ENGINE: {
    headline: "Ephemeris unavailable.",
    cause: "",
  },
};

export function ErrorBlock(props: { code: string; message: string }): JSX.Element {
  const text = () => ERROR_TEXT[props.code] ?? ERROR_TEXT.ENGINE!;
  return (
    <div class="error-block">
      <p class="error-block__headline">{text().headline}</p>
      <p class="error-block__cause">{text().cause || truncate(props.message)}</p>
    </div>
  );
}

function truncate(message: string): string {
  return message.length > 60 ? `${message.slice(0, 59)}…` : message;
}
