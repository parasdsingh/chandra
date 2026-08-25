/**
 * The expanded day detail.
 *
 * The moon view stays inside the v1 field list (docs/DECISIONS.md D-010): phase,
 * illumination, rise and set, nakshatra, rashi, and distance from the Sun, which
 * is here because the grid marks combustion and a mark the day cannot explain is
 * worse than no mark.
 *
 * Every state the grid draws is also named in words here. Degraded readings - no
 * moonrise, circumpolar, a Moshier position - are stated as facts, with no
 * warning colour and no icon: they are normal, not faults.
 */

import type { JSX } from "solid-js";
import { Index, Show } from "solid-js";

import type {
  Combustion,
  DayPanchanga,
  DayDetail as Detail,
  GrahaDay,
  MoonDay,
  Moment,
  TithiSpan,
  TransitEvent,
} from "../ipc/types";
import type { FormatContext } from "../lib/format";
import {
  formatBoundary,
  formatDegrees,
  formatIllumination,
  formatSeparation,
  formatSpan,
  formatSpeed,
  formatTime,
  formatWeekday,
  phaseLabel,
  spokenDegrees,
  spokenSeparation,
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
          {/* Weekday only: the header already carries the date, and printing it
              twice in a 320px panel is the clutter the layout exists to avoid. */}
          {/* The vara completes the second limb at no cost: one word, in the
              row that already names the weekday. Only in a lunar month, where
              it is part of what the day is called. */}
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

          <Show
            when={detail().kind === "moon"}
            fallback={
              <GrahaBody
                detail={detail() as GrahaDay}
                events={props.events}
                grahaName={props.grahaName}
                context={props.context}
              />
            }
          >
            <MoonBody detail={detail() as MoonDay} context={props.context} />
          </Show>

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

function MoonBody(props: { detail: MoonDay; context: FormatContext }): JSX.Element {
  const circumpolar = () => !props.detail.moonrise && !props.detail.moonset;

  return (
    <>
      <div class="detail__headline">
        <span class="detail__phase">{phaseLabel(props.detail.phase)}</span>
        <span class="detail__illum">
          {formatIllumination(props.detail.illumination)}
        </span>
      </div>

      {/* The grid states a tithi number; this is where it is named in words,
          with the boundaries that decide it. A number nobody can check is worse
          than no number at all. */}
      <TithiBlock panchanga={props.detail.panchanga} context={props.context} />

      {/* A missing rise or set is stated on its own row, in the value column.
          Rendering a dash plus a caption underneath added a line and changed the
          shape of the block depending on the day. */}
      <div class="detail__block">
        <SunriseRow
          panchanga={props.detail.panchanga}
          context={props.context}
        />
        <Show
          when={!circumpolar()}
          fallback={
            <Row label="Moon" value="does not rise or set today" muted />
          }
        >
          <RiseRow
            label="Moonrise"
            moment={props.detail.moonrise}
            absent="no rise today"
            context={props.context}
          />
          <RiseRow
            label="Moonset"
            moment={props.detail.moonset}
            absent="no set today"
            context={props.context}
          />
        </Show>
      </div>

      <CombustionBlock combustion={props.detail.combustion} />

      {/* The label names the field once, on the first row. A second span in the
          same day is a continuation of that field, not a new unlabelled field,
          and is dimmed so the one prevailing at sunrise reads first. */}
      <SpanBlock
        label="Nakshatra"
        spans={props.detail.nakshatras.map((span) => ({
          value: `${span.name} · Pada ${span.pada}`,
          caption: formatSpan(span.entry, span.exit, props.context),
          prevailing: span.prevailing,
        }))}
      />

      <SpanBlock
        label="Rashi"
        spans={props.detail.rashis.map((span) => ({
          value: span.name,
          caption: formatSpan(span.entry, span.exit, props.context),
          prevailing: span.prevailing,
        }))}
      />
    </>
  );
}

function GrahaBody(props: {
  detail: GrahaDay;
  events: TransitEvent[];
  grahaName: string;
  context: FormatContext;
}): JSX.Element {
  return (
    <>
      <div class="detail__headline">
        <span class="detail__phase">{props.grahaName}</span>
        {/* The mark every ephemeris uses, next to the name it applies to.
            Carried three more ways below: the Motion row in words, the minus
            sign on the speed, and the event row when a station falls today. */}
        <Show when={props.detail.retrograde}>
          <span class="chip" aria-label="Retrograde">
            ℞
          </span>
        </Show>
      </div>

      <div class="detail__block">
        <Row
          label="Longitude"
          value={formatDegrees(props.detail.degrees_in_rashi)}
          spoken={spokenDegrees(props.detail.degrees_in_rashi)}
        />
        {/* Named in words as well as by the sign on the speed and the chip in
            the headline. A minus sign at 13px is not a state anyone should have
            to infer. */}
        <Row
          label="Motion"
          value={props.detail.retrograde ? "Retrograde" : "Direct"}
        />
        <Row label="Speed" value={formatSpeed(props.detail.speed)} />
      </div>

      <TithiBlock panchanga={props.detail.panchanga} context={props.context} />

      <div class="detail__block">
        <SunriseRow
          panchanga={props.detail.panchanga}
          context={props.context}
        />
        <RiseRow
          label="Rise"
          moment={props.detail.rise}
          absent="no rise today"
          context={props.context}
        />
        <RiseRow
          label="Set"
          moment={props.detail.set}
          absent="no set today"
          context={props.context}
        />
      </div>

      <CombustionBlock combustion={props.detail.combustion} />

      {/* Every graha stands in a nakshatra and a rashi, so the day view is the
          same shape for all nine and for the Moon. */}
      <SpanBlock
        label="Nakshatra"
        spans={props.detail.nakshatras.map((span) => ({
          value: `${span.name} · Pada ${span.pada}`,
          caption: formatSpan(span.entry, span.exit, props.context),
          prevailing: span.prevailing,
        }))}
      />

      <SpanBlock
        label="Rashi"
        spans={props.detail.rashis.map((span) => ({
          value: span.name,
          caption: formatSpan(span.entry, span.exit, props.context),
          prevailing: span.prevailing,
        }))}
      />

      <Show when={props.events.length > 0}>
        <div class="detail__block">
          <Index each={props.events}>
            {(event) => (
              <p class="detail__event">
                <span>{describeEvent(event())}</span>
                <span class="detail__event-time">
                  {formatTime(event().at, props.context)}
                </span>
              </p>
            )}
          </Index>
        </div>
      </Show>
    </>
  );
}

/**
 * Distance from the Sun, and combustion when it applies.
 *
 * Shown for every subject that has an orb, combust or not, so the block keeps
 * one shape and the reading that decides combustion is always on screen. The
 * Sun and the nodes have no orb, so for them the block is absent entirely
 * rather than present and permanently negative.
 *
 * No warning colour: combustion is an ordinary position, not a fault. Nothing
 * in the panel is coloured to mean a state - `--accent` marks today and nothing
 * else - so the reading is in the words.
 */
function CombustionBlock(props: { combustion: Combustion }): JSX.Element {
  return (
    <Show when={props.combustion.orb !== null}>
      <div class="detail__block">
        <Row
          label="From Sun"
          value={formatSeparation(props.combustion.separation)}
          spoken={spokenSeparation(props.combustion.separation)}
        />
        <Show when={props.combustion.combust}>
          <p class="detail__caption">
            Combust — inside the {props.combustion.orb}° orb
          </p>
        </Show>
      </div>
    </Show>
  );
}

/**
 * The tithis touching the day, prevailing one first.
 *
 * Reuses the same block the nakshatra and rashi rows use, so the three read as
 * one list: label on the first row, continuations dimmed, boundaries in the
 * caption. The caption also carries why a number was skipped or repeated, which
 * is the only place in the app those two words appear (DESIGN 11.3).
 */
function TithiBlock(props: {
  panchanga: DayPanchanga | null;
  context: FormatContext;
}): JSX.Element {
  return (
    <Show when={props.panchanga}>
      {(panchanga) => (
        <SpanBlock
          label="Tithi"
          spans={panchanga().tithis.map((span) => ({
            value: `${span.paksha === "shukla" ? "Shukla" : "Krishna"} ${span.name}`,
            caption: tithiCaption(span, props.context),
            prevailing: span.prevailing,
          }))}
        />
      )}
    </Show>
  );
}

/**
 * A tithi's boundaries, and what makes it unusual.
 *
 * An unresolved boundary says so rather than printing a guessed time: the other
 * side is still real and still shown.
 */
function tithiCaption(span: TithiSpan, context: FormatContext): string {
  const window =
    span.entry && span.exit
      ? formatSpan(span.entry, span.exit, context)
      : span.entry
        ? `${formatBoundary(span.entry, context)} → time unavailable`
        : span.exit
          ? `time unavailable → ${formatBoundary(span.exit, context)}`
          : "times unavailable";

  const note =
    span.sunrises === 0
      ? "kshaya, no sunrise"
      : span.sunrises === 2
        ? "two sunrises"
        : "";

  return note ? `${window} · ${note}` : window;
}

/**
 * Sunrise, the instant the day's tithi, nakshatra and rashi are all read at.
 *
 * Without it the numbers in the grid cannot be checked against anything. Where
 * the Sun does not rise the row says so and names local noon, which is the
 * substitute actually used - not a blank and not a dash.
 */
function SunriseRow(props: {
  panchanga: DayPanchanga | null;
  context: FormatContext;
}): JSX.Element {
  return (
    <Show when={props.panchanga}>
      {(panchanga) => (
        <Show
          when={panchanga().sunrise}
          fallback={
            <Row label="Sunrise" value="does not rise; read at noon" muted />
          }
        >
          {(sunrise) => (
            <Row
              label="Sunrise"
              value={formatBoundary(sunrise(), props.context)}
            />
          )}
        </Show>
      )}
    </Show>
  );
}

interface SpanRow {
  value: string;
  caption: string;
  prevailing: boolean;
}

function SpanBlock(props: { label: string; spans: SpanRow[] }): JSX.Element {
  return (
    <div class="detail__block">
      {/* Index, not For: the spans are rebuilt as fresh objects on every read,
          so keying by reference disposes and recreates every row for a value
          that has not changed. The rows are positions in a list, which is what
          Index keys by. */}
      <Index each={props.spans}>
        {(span, index) => (
          <>
            <p
              class="detail__row"
              role="group"
              aria-label={`${props.label}, ${span().value}, ${span().caption}`}
            >
              <span class="detail__label">{index === 0 ? props.label : ""}</span>
              <span
                class="detail__value"
                classList={{ "is-secondary": !span().prevailing }}
              >
                {span().value}
              </span>
            </p>
            <p class="detail__caption">{span().caption}</p>
          </>
        )}
      </Index>
    </div>
  );
}

function Row(props: {
  label: string;
  value: string;
  spoken?: string;
  muted?: boolean;
}): JSX.Element {
  return (
    <p
      class="detail__row"
      role="group"
      aria-label={`${props.label}, ${props.spoken ?? props.value}`}
    >
      <span class="detail__label">{props.label}</span>
      <span class="detail__value" classList={{ "is-absent": props.muted }}>
        {props.value}
      </span>
    </p>
  );
}

/**
 * A rise or set row.
 *
 * When the event does not occur, the value column carries the reason instead of
 * a time. There is no dash and no extra caption line: the block keeps the same
 * shape whether or not the Moon rose.
 */
function RiseRow(props: {
  label: string;
  moment: Moment | null;
  absent: string;
  context: FormatContext;
}): JSX.Element {
  const text = () =>
    props.moment ? formatBoundary(props.moment, props.context) : props.absent;

  return (
    <p class="detail__row" role="group" aria-label={`${props.label}, ${text()}`}>
      <span class="detail__label">{props.label}</span>
      <span class="detail__value" classList={{ "is-absent": !props.moment }}>
        {text()}
      </span>
    </p>
  );
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

function ErrorBlock(props: { code: string; message: string }): JSX.Element {
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
