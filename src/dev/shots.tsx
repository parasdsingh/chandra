/**
 * Release screenshots. Development only, at `?preview&shots`.
 *
 * The landing page needs pictures of the app, and a picture of an app should be
 * the app. These are the shipping components rendered from the same fixture the
 * visual harness uses - not mockups, not a redraw, and not a photograph of a
 * screen with a cursor in it.
 *
 * Its own mode rather than a corner of the harness for two reasons. The harness
 * draws forty panels with animations running in most of them, which is heavy
 * enough that a browser driving it for a screenshot times out; and a screenshot
 * wants one panel, isolated, on a plain ground, at a size worth publishing.
 *
 * Each panel is drawn at `--panel-scale: 1.6`. The stage that reserves its room
 * is 512x532 points; captured on a 2x display that is 1024x1062 pixels, which
 * is what every committed asset measures and what `site/index.html` declares as
 * the intrinsic width.
 *
 * `id` is the asset's filename: shot `chart` is `site/assets/chart.png`. The
 * two were mapped by hand and did not correspond - `grid` was published as
 * `degree-grid.png` - so a shot and the picture on the page could drift apart
 * with nothing to notice.
 */

import type { JSX } from "solid-js";
import { For } from "solid-js";

import { ChartView } from "../components/ChartView";
import { DayDetail } from "../components/DayDetail";
import { Header } from "../components/Header";
import { MonthCells, WeekdayRow } from "../components/MonthGrid";
import { SettingsView } from "../components/SettingsView";
import type {
  Chakra,
  DayDetail as Detail,
  GrahaInfo,
  GrahaMonth,
  MoonMonth,
  Snapshot,
} from "../ipc/types";
import fixture from "./fixture.json";
import { previewBoot } from "./preview";

const data = fixture as unknown as {
  timeZone: string;
  grahas: GrahaInfo[];
  moonMonth: MoonMonth;
  moonDay: Detail;
  grahaMonth: GrahaMonth;
  lunarGrahaDay: Detail;
  snapshot: Snapshot;
  chakra: Chakra;
  chakraNavamsa: Chakra;
};

/** The scale every shot is drawn at.
 *
 *  1.6, not 2: at 2 the stage is 640x664 points and the panel no longer fits
 *  the capture viewport beside its caption. The committed assets are 1024x1062
 *  pixels, which is 320x332 at 1.6 on a 2x display - no upscaling anywhere. The
 *  panel is vector throughout, so a fractional scale costs nothing.
 *
 *  This said "Two" over the 1.6, and the header said the image was about 830
 *  pixels wide. Both were left from the earlier value. */
const SHOT_SCALE = 1.6;

/// The bootstrap the panes are drawn against.
///
/// Imported, not taken as a prop. `App.tsx` was importing it from `preview` and
/// passing it in, and this module already imports `fixture.json` from the same
/// place - so the prop was a parameter with exactly one possible value.
export function Shots(): JSX.Element {
  const shots: { id: string; caption: string; view: () => JSX.Element }[] = [
    {
      id: "chart",
      caption: "The Lagna Kundali",
      view: () => (
        <>
          <Chrome title={`${data.chakra.lagna.name} Lagna`} division={1} view="chart" />
          <div class="region">
            <ChartView
              chart={data.chakra}
              error={undefined}
              format="north"
              numbered={false}
              timeZone={data.timeZone}
              animate={false}
              grid={false}
              sky
            />
          </div>
        </>
      ),
    },
    {
      id: "degree-grid",
      caption: "The degree grid",
      view: () => (
        <>
          <Chrome title={`${data.chakra.lagna.name} Lagna`} division={1} view="chart" />
          <div class="region">
            <ChartView
              chart={data.chakra}
              error={undefined}
              format="north"
              numbered={false}
              timeZone={data.timeZone}
              animate={false}
              grid
              sky
            />
          </div>
        </>
      ),
    },
    {
      id: "navamsa",
      caption: "The navamsa, the same instant",
      view: () => (
        <>
          <Chrome title={`${data.chakraNavamsa.lagna.name} Lagna`} division={9} view="chart" />
          <div class="region">
            <ChartView
              chart={data.chakraNavamsa}
              error={undefined}
              format="north"
              numbered={false}
              timeZone={data.timeZone}
              animate={false}
              grid={false}
              sky
            />
          </div>
        </>
      ),
    },
    {
      id: "calendar",
      caption: "The month",
      view: () => (
        <>
          <Chrome title="August 2026" division={1} view="calendar" />
          <div class="region">
            <WeekdayRow firstWeekday={0} />
            <div class="grid-region">
              <MonthCells
                firstWeekday={0}
                month={data.moonMonth}
                kind="moon"
                selected={data.moonDay.date}
                today={data.moonDay.date}
                southern={false}
                active
                ingress="rashi"
                onSelect={() => {}}
              />
            </div>
          </div>
        </>
      ),
    },
    {
      id: "graha-month",
      caption: "A graha's month",
      view: () => (
        <>
          <Chrome title={data.grahaMonth.label} division={1} view="calendar" subject="mangala" />
          <div class="region">
            <WeekdayRow firstWeekday={0} />
            <div class="grid-region">
              <MonthCells
                firstWeekday={0}
                month={data.grahaMonth}
                kind="graha"
                info={data.grahas.find((graha) => graha.key === "mangala")}
                selected={null}
                today={data.moonDay.date}
                southern={false}
                active
                ingress="rashi"
                onSelect={() => {}}
              />
            </div>
          </div>
        </>
      ),
    },
    {
      id: "retrograde",
      caption: "A retrograde day",
      view: () => (
        <>
          <Chrome title="21 August 2026" division={1} view="day" subject="shani" />
          <div class="region">
            <DayDetail
              detail={data.lunarGrahaDay}
              events={[]}
              context={{ timeZone: data.timeZone }}
              isToday={false}
              lunar
              error={undefined}
              onStep={() => {}}
            />
          </div>
        </>
      ),
    },
    {
      id: "day",
      caption: "A day",
      view: () => (
        <>
          <Chrome title="20 August 2026" division={1} view="day" />
          <div class="region">
            <DayDetail
              detail={data.moonDay}
              events={[]}
              context={{ timeZone: data.timeZone }}
              isToday
              lunar={false}
              error={undefined}
              onStep={() => {}}
            />
          </div>
        </>
      ),
    },
    {
      id: "settings",
      caption: "Settings",
      view: () => (
        <>
          <Chrome title="Settings" division={1} view="settings" />
          <div class="region">
            <SettingsView
              boot={previewBoot}
              section="root"
              onOpen={() => {}}
              apply={() => {}}
            />
          </div>
        </>
      ),
    },
  ];

  return (
    <div class="shots">
      <For each={shots}>
        {(shot) => (
          <figure class="shots__shot" id={`shot-${shot.id}`}>
            <div class="shots__stage">
              <div
                class="panel-frame"
                style={{ "--panel-scale": String(SHOT_SCALE) }}
              >
                <div class="panel is-opaque">{shot.view()}</div>
              </div>
            </div>
            <figcaption>{shot.caption}</figcaption>
          </figure>
        )}
      </For>
    </div>
  );
}

/** The header, which every view has and no shot should be missing. */
function Chrome(props: {
  title: string;
  division: number;
  view: "calendar" | "day" | "settings" | "chart";
  /** Which calendar this shot is of. The Moon unless said otherwise - the other
   *  eight put their own glyph in the header and their own name on the tray. */
  subject?: "chandra" | "mangala" | "shani";
}): JSX.Element {
  const subject = () => props.subject ?? "chandra";
  const named: Record<string, string> = {
    chandra: "Chandra",
    mangala: "Mangala",
    shani: "Shani",
  };
  return (
    <Header
      division={props.division}
      subject={subject()}
      subjectName={named[subject()]!}
      info={data.grahas.find((graha) => graha.key === subject())}
      snapshot={data.snapshot}
      southern={false}
      title={props.title}
      adhika={false}
      selected={data.moonDay.date}
      view={props.view}
      onBack={() => {}}
      onSettings={() => {}}
      jumping={false}
      onJump={() => {}}
      gated={false}
    />
  );
}
