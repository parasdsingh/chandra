/**
 * The expanded day detail.
 *
 * The moon view shows exactly the six v1 fields and nothing else
 * (docs/DECISIONS.md D-010). Degraded readings - no moonrise, circumpolar, a
 * Moshier position - are stated as facts in words, with no warning colour and no
 * icon: they are normal, not faults.
 */

import type { JSX } from "solid-js";
import { For, Show } from "solid-js";

import type {
  DayDetail as Detail,
  GrahaDay,
  MoonDay,
  Moment,
  TransitEvent,
} from "../ipc/types";
import type { FormatContext } from "../lib/format";
import {
  formatBoundary,
  formatDegrees,
  formatFullDate,
  formatIllumination,
  formatSpan,
  formatSpeed,
  formatTime,
  phaseLabel,
  spokenDegrees,
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
          <div class="detail__date">
            <Show when={props.isToday}>
              <span>TODAY · </span>
            </Show>
            {formatFullDate(detail().date)}
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

      <div class="detail__block">
        <RiseRow
          label="Moonrise"
          moment={props.detail.moonrise}
          context={props.context}
          note={
            circumpolar()
              ? undefined
              : props.detail.moonrise
                ? undefined
                : "no rise on this date"
          }
        />
        <RiseRow
          label="Moonset"
          moment={props.detail.moonset}
          context={props.context}
          note={
            circumpolar()
              ? undefined
              : props.detail.moonset
                ? undefined
                : "no set on this date"
          }
        />
        {/* One caption for the pair, worded differently from a skipped rise so
            the two are never conflated. */}
        <Show when={circumpolar()}>
          <p class="detail__note">always above or below the horizon</p>
        </Show>
      </div>

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
        {/* Retrograde is carried three ways: the chip, the explicit minus sign
            on the speed, and the event row when a station falls today. */}
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
        <Row label="Speed" value={formatSpeed(props.detail.speed)} />
      </div>

      <div class="detail__block">
        <RiseRow label="Rise" moment={props.detail.rise} context={props.context} />
        <RiseRow label="Set" moment={props.detail.set} context={props.context} />
      </div>

      <Show when={props.events.length > 0}>
        <div class="detail__block">
          <For each={props.events}>
            {(event) => (
              <p class="detail__event">
                <span>{describeEvent(event)}</span>
                <span class="detail__event-time">
                  {formatTime(event.at, props.context)}
                </span>
              </p>
            )}
          </For>
        </div>
      </Show>
    </>
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
      <For each={props.spans}>
        {(span, index) => (
          <>
            <p
              class="detail__row"
              role="group"
              aria-label={`${props.label}, ${span.value}, ${span.caption}`}
            >
              <span class="detail__label">{index() === 0 ? props.label : ""}</span>
              <span
                class="detail__value"
                classList={{ "is-secondary": !span.prevailing }}
              >
                {span.value}
              </span>
            </p>
            <p class="detail__caption">{span.caption}</p>
          </>
        )}
      </For>
    </div>
  );
}

function Row(props: { label: string; value: string; spoken?: string }): JSX.Element {
  return (
    <p class="detail__row" role="group" aria-label={`${props.label}, ${props.spoken ?? props.value}`}>
      <span class="detail__label">{props.label}</span>
      <span class="detail__value">{props.value}</span>
    </p>
  );
}

function RiseRow(props: {
  label: string;
  moment: Moment | null;
  context: FormatContext;
  note?: string;
}): JSX.Element {
  return (
    <>
      <p
        class="detail__row"
        role="group"
        aria-label={
          props.moment
            ? `${props.label}, ${formatBoundary(props.moment, props.context)}`
            : `${props.label}, none${props.note ? `, ${props.note}` : ""}`
        }
      >
        <span class="detail__label">{props.label}</span>
        <span class="detail__value" classList={{ "is-absent": !props.moment }}>
          <Show when={props.moment} fallback="—">
            {(moment) => formatBoundary(moment(), props.context)}
          </Show>
        </span>
      </p>
      <Show when={props.note}>
        <p class="detail__note">{props.note}</p>
      </Show>
    </>
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
