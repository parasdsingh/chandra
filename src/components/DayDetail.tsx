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
  MoonDay,
  NakshatraSpan,
  RashiSpan,
  TithiSpan,
  TransitEvent,
} from "../ipc/types";
import type { FormatContext } from "../lib/format";
import {
  formatTime,
  formatUntil,
  formatWeekday,
  phaseLabel,
} from "../lib/format";
import { describeEvent } from "./MonthGrid";

interface Props {
  detail: Detail | undefined;
  events: TransitEvent[];
  grahaName: string;
  context: FormatContext;
  isToday: boolean;
  error: { code: string; message: string } | undefined;
}

export function DayDetail(props: Props): JSX.Element {
  return (
    <div class="detail" role="region" aria-live="polite" aria-label="Day detail">
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
            <Show when={detail().panchanga}>
              {(panchanga) => <span> · {panchanga().vara_name}</span>}
            </Show>
          </div>

          <Subject
            detail={detail()}
            events={props.events}
            grahaName={props.grahaName}
            context={props.context}
          />

          <Show when={detail().source === "moshier"}>
            <p class="detail__provenance">
              Moshier ephemeris — reduced precision outside 1800–2399
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
  grahaName: string;
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

  /** The subject's own rise, named for the subject. */
  const riseLabel = () => {
    if (moon()) return "Moonrise";
    return graha()!.graha === "surya" ? "Sunrise" : "Rise";
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
      <Show when={props.detail.combustion.combust}>
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
