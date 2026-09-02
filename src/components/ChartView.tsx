/**
 * The chart pane: the caption that places the chart, the chart, and the note
 * that says when the chart is approximate.
 *
 * Its own component so the panel and the visual harness draw the same pane from
 * the same source. Built inline in `Panel` it could only ever be looked at by
 * opening the menu bar, and a pane the harness re-created would be a second
 * copy free to drift from the first.
 *
 * Takes a resolved reading and an already-normalised error, the same shape
 * `DayDetail` takes, so the caller keeps its resource handling and this keeps
 * the drawing.
 */

import type { JSX } from "solid-js";
import { Show } from "solid-js";

import type { Chakra as ChakraData, ChartFormat } from "../ipc/types";
import { formatTime } from "../lib/format";
import { Chakra } from "./Chakra";
import { ErrorBlock } from "./DayDetail";

interface Props {
  /** The latest reading, kept on screen while the next one is fetched. */
  chart: ChakraData | undefined;
  error: { code: string; message: string } | undefined;
  format: ChartFormat;
  numbered: boolean;
  timeZone: string;
}

export function ChartView(props: Props): JSX.Element {
  return (
    <div class="chakra-view">
      <Show
        when={props.chart}
        fallback={
          <Show when={props.error}>
            {(problem) => (
              <ErrorBlock code={problem().code} message={problem().message} />
            )}
          </Show>
        }
      >
        {(data) => (
          <>
            {/* One line, and it places the chart rather than translating its
                title. The degree first because it is the most volatile figure
                in the view - the lagna moves one degree every four minutes -
                then the instant, then the place, which is what makes the degree
                mean anything: a longitude 300km out moves the lagna about three
                degrees, and both reference applications print the place beside
                every chart.

                It drops right to left as the line narrows, except that the
                place is the last thing to go: if the place alone will not fit,
                the other two come back instead. */}
            {/* The scheme is on the hover rather than in the line. A D9
                computed one way looks exactly like a D9 computed another, so a
                chart that cannot say what produced it invites the reader to
                assume it matches whatever they last saw elsewhere (D-032) - but
                the panel is 318 points wide, and this line already carries the
                three things that place the chart. The caption is where the
                chart's provenance lives, so the scheme goes on it. */}
            <p class="chakra__gloss" title={provenance(data())}>
              {caption(data(), props.timeZone)}
            </p>

            <Chakra
              data={data()}
              format={props.format}
              numbered={props.numbered}
            />

            {/* The chart is as exact as the ephemeris behind it, and outside
                1800-2399 that is the analytic fallback. Every other view says
                so; this one did not, which made it the one surface that could
                print a degree it had not earned. */}
            <Show when={data().source === "moshier"}>
              <p class="detail__provenance">
                Outside 1800–2399. Positions here are approximate.
              </p>
            </Show>
          </>
        )}
      </Show>
    </div>
  );
}

/**
 * The line under the chart's title.
 *
 * A ladder, like the header's. The degree is dropped first, then the instant;
 * the place is dropped only if it is what will not fit, and then the other two
 * come back rather than leaving the line nearly empty.
 */
function caption(chart: ChakraData, timeZone: string): string {
  const [degrees, minutes] = chart.lagna.degrees_in_rashi;
  const at = `${degrees}°${String(minutes).padStart(2, "0")}′`;
  const clock = formatTime(
    { unix_ms: chart.unix_ms, day_offset: 0 },
    { timeZone },
  );
  const place = chart.place.toUpperCase();

  return (
    [
      `${at} · ${clock} · ${place}`,
      `${clock} · ${place}`,
      place,
      `${at} · ${clock}`,
    ].find((line) => line.length <= CAPTION_LIMIT) ?? at
  );
}

/**
 * What produced this chart, for the hover.
 *
 * Spelled out rather than abbreviated: `D9` is the name of the division to
 * somebody who already knows, and this exists for somebody who does not.
 */
function provenance(chart: ChakraData): string {
  const division =
    chart.varga === "d1"
      ? "Rashi chart (D1)"
      : `${VARGA_NAMES[chart.varga]} (${chart.varga.toUpperCase()})`;
  return `${division} · ${chart.scheme} scheme`;
}

const VARGA_NAMES: Record<string, string> = {
  d1: "Rashi",
  d3: "Drekkana",
  d7: "Saptamsa",
  d9: "Navamsa",
  d12: "Dwadasamsa",
};

/** Characters that fit the caption's 288px line at 10px uppercase.
 *
 * Measured rather than assumed: the micro type is 10px and its tracking 0.6px,
 * so an average uppercase glyph runs about 7px and 288 holds about forty. */
const CAPTION_LIMIT = 40;
