/**
 * Visual harness. Development only.
 *
 * Renders the panel's views against real almanac output (`fixture.json`, from
 * `make preview`), so layout, spacing, glyphs and degraded states can be
 * inspected without a menu bar. It touches no IPC and is reachable only at
 * `?preview` in a dev build.
 */

import type { JSX } from "solid-js";
import { createSignal, For, onCleanup } from "solid-js";

import fixture from "./fixture.json";
import { Chakra as ChakraChart } from "../components/Chakra";
import { DayDetail } from "../components/DayDetail";
import { ChartView } from "../components/ChartView";
import { Header } from "../components/Header";
import { MonthCells, WeekdayRow } from "../components/MonthGrid";
import { PhaseGlyph } from "../components/PhaseGlyph";
import { GrahaGlyph } from "../components/GrahaGlyph";
import {
  SECTION_TITLES,
  SettingsView,
  type SettingsSection,
} from "../components/SettingsView";
import type {
  Bootstrap,
  Chakra,
  ChartFormat,
  DayDetail as Detail,
  GrahaInfo,
  GrahaMonth,
  MoonMonth,
  Snapshot,
} from "../ipc/types";
import type { FormatContext } from "../lib/format";

const data = fixture as unknown as {
  timeZone: string;
  grahas: GrahaInfo[];
  moonMonth: MoonMonth;
  lunarMonth: MoonMonth;
  moonDay: Detail;
  moonDayNoRise: Detail;
  lunarGrahaDay: Detail;
  polarDay: Detail;
  grahaMonth: GrahaMonth;
  grahaDay: Detail;
  moshierDay: Detail;
  snapshot: Snapshot;
  chakra: Chakra;
  chakraCrowded: Chakra;
  chakraNavamsa: Chakra;
  chakraNext: Chakra;
  chakraHora: Chakra;
  chakraTrimsamsa: Chakra;
  chakraCorner: Chakra;
  chakraWall: Chakra;
  chakraShashtiamsa: Chakra;
  chakraShashtiamsaNext: Chakra;
  chakraMoshier: Chakra;
};

const context: FormatContext = { timeZone: data.timeZone };

const boot: Bootstrap = {
  settings: {
    schema_version: 13,
    location: {
      mode: "manual",
      place: {
        label: "Bengaluru",
        zone: "Asia/Kolkata",
        latitude: 12.9716,
        longitude: 77.5946,
        elevation: 920,
      },
      elevation: null,
    },
    sidereal: { ayanamsa: "lahiri", node_type: "true" },
    calendar: { month_system: "amanta", ingress: "rashi" },
    panchanga: { yogas: true, karanas: true, muhurtas: true },
    chart: {
      format: "north",
      vargas: ["d1", "d9"],
      numbered: false,
      animate: true,
      grid: false,
      sky: true,
    },
    tray: { subjects: ["chandra", "mangala", "shani"], colour_mode: false },
    appearance: { scale: 1 },
  },
  location: {
    label: "Bengaluru",
    zone: "Asia/Kolkata",
    latitude: 12.9716,
    longitude: 77.5946,
    elevation: 920,
    elevation_known: true,
    provenance: "manual",
  },
  subject: "chandra",
  subjects: ["mangala", "shani"],
  // The harness runs in a browser tab with no AppKit material behind it, so the
  // panel paints its own ground here exactly as it does when vibrancy fails.
  panel_material: false,
  library_version: "2.10.03",
  ayanamsas: [
    { key: "lahiri", label: "Lahiri (Chitrapaksha)" },
    { key: "raman", label: "Raman" },
    { key: "krishnamurti", label: "Krishnamurti (KP)" },
    { key: "true_chitra", label: "True Chitra" },
  ],
  node_types: [
    { key: "true", label: "True node" },
    { key: "mean", label: "Mean node" },
  ],
  month_systems: [
    { key: "solar", label: "Solar (Gregorian)" },
    { key: "amanta", label: "Lunar, amanta (new moon)" },
    { key: "purnimanta", label: "Lunar, purnimanta (full moon)" },
  ],
  grahas: data.grahas,
  // The harness serves what the back end serves. Kept short rather than all
  // sixteen: the settings pane is what draws them, and it draws whatever this
  // list holds.
  vargas: [
    { key: "d1", label: "D1 · Rashi", name: "Rashi", division: 1 },
    { key: "d2", label: "D2 · Hora", name: "Hora", division: 2 },
    { key: "d9", label: "D9 · Navamsa", name: "Navamsa", division: 9 },
    { key: "d30", label: "D30 · Trimsamsa", name: "Trimsamsa", division: 30 },
  ] as const,
};

/**
 * One chart, in the panel it is drawn in.
 *
 * The header comes with it rather than the chart alone: the title is built from
 * the reading - the rising sign, not the feature's name - so a chart whose
 * header says the wrong thing is only visible when both are on screen together.
 */
/**
 * The handover, repeating.
 *
 * Two real charts two hours apart, swapped every few seconds. Nothing else in
 * the harness has a handover in it: every other case is one instant, and a
 * handover is a change between two.
 */
function HandoverCase(): JSX.Element {
  const [ahead, setAhead] = createSignal(false);
  const timer = setInterval(() => setAhead((was) => !was), 2600);
  onCleanup(() => clearInterval(timer));

  return (
    <ChartCase
      title="Chart · the handover, every 2.6s"
      chart={ahead() ? data.chakraNext : data.chakra}
      format="north"
      numbered={false}
      animate
    />
  );
}

function ChartCase(props: {
  title: string;
  chart: Chakra | undefined;
  format: ChartFormat;
  numbered: boolean;
  error?: { code: string; message: string };
  animate?: boolean;
  grid?: boolean;
  sky?: boolean;
}): JSX.Element {
  return (
    <Case title={props.title}>
      <Header
        division={props.chart ? Number(props.chart.varga.slice(1)) : 1}
        subject="chandra"
        subjectName="Chandra"
        info={data.grahas.find((graha) => graha.key === "chandra")}
        snapshot={data.snapshot}
        southern={false}
        title={props.chart ? `${props.chart.lagna.name} Lagna` : ""}
        adhika={false}
        selected={null}
        view="chart"
        onBack={() => {}}
        onSettings={() => {}}
        jumping={false}
        onJump={() => {}}
        gated={false}
      />
      <div class="region">
        <ChartView
          chart={props.chart}
          error={props.error}
          format={props.format}
          numbered={props.numbered}
          timeZone={data.timeZone}
          // The harness draws every chart at the position the static layout
          // gives it. The drift is stepped by its own case below, where it can
          // be checked at more than one instant.
          animate={props.animate ?? false}
          grid={props.grid ?? false}
          // Off in the harness by default. Every geometric check below reads
          // the chart's own elements, and a hundred and twenty circles behind
          // them is a hundred and twenty things to skip; the sky gets its own
          // case instead, where it is the subject.
          sky={props.sky ?? false}
        />
      </div>
    </Case>
  );
}

/**
 * One chart drawn by the pathway prototype, without the pane around it.
 *
 * `ChartView` is not in the way here on purpose. The prototype is a change to
 * where bodies stand inside a compartment and to nothing else, so the panel's
 * own pane has no part in it and is left exactly as it ships - the harness
 * reaches past it to the renderer, which is the only thing being looked at.
 *
 * `docs/design/traversal.md`.
 */
function PathwayCase(props: {
  title: string;
  chart: Chakra;
  progress: number;
  grid?: boolean;
  /** Whether each body eases to its next position rather than stepping to it.
   *  Only the running case sets this: the stepped cases are measured by the
   *  geometric checks, and a body caught mid-slide is not at the position the
   *  layout gave it. */
  animate?: boolean;
}): JSX.Element {
  return (
    <Case title={props.title}>
      <div class="region">
        <div class="chakra-view">
          <ChakraChart
            data={props.chart}
            format="north"
            numbered={false}
            progress={props.progress}
            animate={props.animate ?? false}
            grid={props.grid}
          />
        </div>
      </div>
    </Case>
  );
}

/**
 * The pathway, running.
 *
 * The static cases below draw five instants; this draws all of them. One
 * crossing of the rising sign takes five seconds here against two hours in D1,
 * which is the only way the traversal can be judged as motion rather than as a
 * strip of positions.
 */
function PathwayRun(props: {
  title: string;
  chart: Chakra;
  grid?: boolean;
}): JSX.Element {
  const [at, setAt] = createSignal(0);
  const timer = setInterval(() => setAt((was) => (was + 0.02) % 1), 100);
  onCleanup(() => clearInterval(timer));

  return (
    <PathwayCase
      title={props.title}
      chart={props.chart}
      progress={at()}
      grid={props.grid}
      // The only case that exercises the smoothing. Everything else here is
      // stepped and measured.
      animate
    />
  );
}

/**
 * Every crowding case at one progress, with the progress under the reader's
 * hand.
 *
 * The geometric check is the reason this exists rather than a longer strip of
 * fixed charts. An arrangement that is legal at 0% and at 100% can be illegal
 * at 43%, so the check has to walk the whole crossing - and it can, by driving
 * this one slider and re-measuring at every step. Four charts is the whole
 * range of shapes the layout has to survive: a kite holding eight, a corner
 * triangle holding seven, a wall triangle holding seven, and the hora, which
 * puts eight in one compartment on any day of the year.
 */
function PathwaySweep(): JSX.Element {
  const [at, setAt] = createSignal(50);
  const [grid, setGrid] = createSignal(false);

  const charts: [string, Chakra][] = [
    ["D1", data.chakra],
    ["eight in a kite", data.chakraCrowded],
    ["seven in a corner triangle", data.chakraCorner],
    ["seven in a wall triangle", data.chakraWall],
    ["D2 hora", data.chakraHora],
    ["D9 navamsa", data.chakraNavamsa],
    ["D30 trimsamsa", data.chakraTrimsamsa],
    ["D60 shashtiamsa", data.chakraShashtiamsa],
  ];

  return (
    <section class="preview__case preview__case--wide" id="pathway-sweep">
      <h2 class="preview__title">Pathway · the crossing, swept</h2>
      <div class="preview__controls">
        <label>
          progress
          <input
            id="pathway-progress"
            type="range"
            min="0"
            max="100"
            value={at()}
            onInput={(event) => setAt(Number(event.currentTarget.value))}
          />
          <output id="pathway-progress-value">{at()}%</output>
        </label>
        <label>
          <input
            id="pathway-grid"
            type="checkbox"
            checked={grid()}
            onChange={(event) => setGrid(event.currentTarget.checked)}
          />
          degree grid
        </label>
      </div>
      <div class="preview__row">
        <For each={charts}>
          {([label, chart]) => (
            <div class="preview__cell">
              <span class="preview__caption">{label}</span>
              <div class="panel-frame">
                <div class="panel">
                  <div class="region">
                    <div class="chakra-view">
                      <ChakraChart
                        data={chart}
                        format="north"
                        numbered={false}
                        progress={at() / 100}
                        animate={false}
                        grid={grid()}
                      />
                    </div>
                  </div>
                </div>
              </div>
            </div>
          )}
        </For>
      </div>
    </section>
  );
}

function Case(props: { title: string; children: JSX.Element }): JSX.Element {
  return (
    <section class="preview__case">
      <h2 class="preview__title">{props.title}</h2>
      <div class="panel-frame">
        <div class="panel">{props.children}</div>
      </div>
    </section>
  );
}

function moonHeader(title: string) {
  return (
    <Header
      division={1}
      subject="chandra"
      subjectName="Chandra"
      info={data.grahas.find((graha) => graha.key === "chandra")}
      snapshot={data.snapshot}
      southern={false}
      title={title}
      adhika={false}
      selected={null}
      view="calendar"
      onBack={() => {}}
      onSettings={() => {}}
      jumping={false}
      onJump={() => {}}
      gated={false}
    />
  );
}

export function Preview(): JSX.Element {
  // The harness renders every settings section at once, so navigation between
  // them is inert here.
  const [, setSection] = createSignal<SettingsSection>("root");

  return (
    <div class="preview">
      <Case title="Moon · solar month">
        {moonHeader(data.moonMonth.label)}
        <div class="region">
          <div class="grid-region">
            <WeekdayRow firstWeekday={0} />
            <MonthCells
              firstWeekday={0}
              month={data.moonMonth}
              kind="moon"
              selected={{ year: 2026, month: 8, day: 21 }}
              today={{ year: 2026, month: 8, day: 21 }}
              southern={false}
              ingress="rashi"
              active
              onSelect={() => {}}
            />
          </div>
        </div>
      </Case>

      <Case title="Moon · lunar month (amanta)">
        {moonHeader(data.lunarMonth.label)}
        <div class="region">
          <div class="grid-region">
            <WeekdayRow firstWeekday={0} />
            <MonthCells
              firstWeekday={0}
              month={data.lunarMonth}
              kind="moon"
              selected={null}
              today={{ year: 2026, month: 8, day: 21 }}
              southern={false}
              ingress="rashi"
              active
              onSelect={() => {}}
            />
          </div>
        </div>
      </Case>

      <Case title="Moon · day view">
        <Header
          division={1}
          subject="chandra"
          subjectName="Chandra"
          info={data.grahas.find((graha) => graha.key === "chandra")}
          snapshot={data.snapshot}
          southern={false}
          title=""
          adhika={false}
          selected={{ year: 2026, month: 8, day: 20 }}
          view="day"
          onBack={() => {}}
          onSettings={() => {}}
          jumping={false}
          onJump={() => {}}
          gated={false}
        />
        <div class="region">
          <DayDetail
            detail={data.moonDay}
            events={[]}
            context={context}
            isToday
            lunar={false}
            error={undefined}
            // Wired here so the harness draws the day steps. The other day
            // cases leave it off, which is also worth seeing: the row keeps its
            // shape and the name stays centred with no arrows in it.
            onStep={() => {}}
          />
        </div>
      </Case>

      {/* The same reading with `lunar` set, which is the only thing that
          differs: a lunar day is the same day, named by its tithi. The fixture
          used to carry a second copy of it under another key, computed by an
          identical call - so the harness looked like it covered two states and
          covered one. */}
      <Case title="Moon · day view, lunar month">
        <div class="region">
          <DayDetail
            detail={data.moonDay}
            events={[]}
            context={context}
            isToday={false}
            lunar={true}
            error={undefined}
          />
        </div>
      </Case>

      {/* A graha in a lunar month: the one view shape nothing else here covers.
          It was in the fixture and drawn nowhere, which is the same gap as the
          duplicate above wearing the opposite disguise. */}
      <Case title="Shani · day view, lunar month">
        <div class="region">
          <DayDetail
            detail={data.lunarGrahaDay}
            events={[]}
            context={context}
            isToday={false}
            lunar={true}
            error={undefined}
          />
        </div>
      </Case>

      <Case title="Moon · a day with no moonrise">
        <div class="region">
          <DayDetail
            detail={data.moonDayNoRise}
            events={[]}
            context={context}
            isToday={false}
            lunar={true}
            error={undefined}
          />
        </div>
      </Case>

      {/* Longyearbyen in December: the Sun does not rise, so the day cannot be
          named after the tithi at sunrise and local noon is used instead. The
          payload has carried that distinction since the rule was written and
          nothing drew it, which is what this case exists to keep true. */}
      <Case title="Moon · polar night, named for local noon">
        <div class="region">
          <DayDetail
            detail={data.polarDay}
            events={[]}
            context={{ timeZone: "Arctic/Longyearbyen" }}
            isToday={false}
            lunar={true}
            error={undefined}
          />
        </div>
      </Case>

      <Case title="Moon · reduced precision (1650)">
        <div class="region">
          <DayDetail
            detail={data.moshierDay}
            events={[]}
            context={context}
            isToday={false}
            lunar={true}
            error={undefined}
          />
        </div>
      </Case>

      <Case title="Mangala · retrograde month">
        <Header
          division={1}
          subject="mangala"
          subjectName="Mangala"
          info={data.grahas.find((graha) => graha.key === "mangala")}
          snapshot={undefined}
          southern={false}
          title={data.grahaMonth.label}
          adhika={false}
          selected={null}
          view="calendar"
          onBack={() => {}}
          onSettings={() => {}}
          jumping={false}
          onJump={() => {}}
          gated={false}
        />
        <div class="region">
          <div class="grid-region">
            <WeekdayRow firstWeekday={0} />
            <MonthCells
              firstWeekday={0}
              month={data.grahaMonth}
              kind="graha"
              info={data.grahas.find((graha) => graha.key === "mangala")}
              selected={{ year: 2025, month: 2, day: 24 }}
              today={{ year: 2025, month: 2, day: 10 }}
              southern={false}
              ingress="rashi"
              active
              onSelect={() => {}}
            />
          </div>
        </div>
      </Case>

      <Case title="Mangala · station day">
        <div class="region">
          <DayDetail
            detail={data.grahaDay}
            events={data.grahaMonth.events.filter(
              (event) => event.date.day === 24,
            )}
            context={context}
            isToday={false}
            lunar={true}
            error={undefined}
          />
        </div>
      </Case>

      <Case title="Error state">
        <div class="region">
          <DayDetail
            detail={undefined}
            events={[]}
            context={context}
            isToday={false}
            lunar={true}
            error={{
              code: "INVALID_DATE",
              message: "1650-08-20 is outside the range Chandra has data for",
            }}
          />
        </div>
      </Case>

      <For
        each={
          // Every section the pane can show. `chart` was missing, which is how
          // its division rows shipped with three children against a four-column
          // grid: the surface was not drawn here, so nothing showed it.
          [
            "root",
            "calendar",
            "location",
            "astrology",
            "chart",
            "motion",
            "grid",
            "menubar",
            "advanced",
            "size",
            "about",
          ] as const
        }
      >
        {(id) => (
          <Case title={`Settings · ${id}`}>
            <Header
              division={1}
              subject="chandra"
              subjectName="Chandra"
              info={data.grahas.find((graha) => graha.key === "chandra")}
              snapshot={data.snapshot}
              southern={false}
              title={SECTION_TITLES[id]}
              adhika={false}
              selected={null}
              view="settings"
              onBack={() => {}}
              onSettings={() => {}}
              jumping={false}
              onJump={() => {}}
              gated={false}
            />
            <div class="region">
              <SettingsView
                boot={boot}
                section={id}
                onOpen={setSection}
                apply={() => {}}
              />
            </div>
          </Case>
        )}
      </For>

      {/* Every chart format, drawn from one reading. What differs between them
          is where a rashi is put on screen and what is written in the
          compartment; none of them differs in what is true, so a bug that
          shows in only one of the three is a drawing bug and belongs here. */}
      <For
        each={
          [
            ["North Indian", "north", false],
            ["North Indian · numbered", "north", true],
            ["South Indian", "south", false],
            ["East Indian", "east", false],
          ] as const
        }
      >
        {([title, format, numbered]) => (
          <ChartCase
            title={`Chart · ${title}`}
            chart={data.chakra}
            format={format}
            numbered={numbered}
          />
        )}
      </For>

      {/* The navamsa of the same instant the case above draws in D1. Side by
          side, because what is worth checking about a division is that it is a
          different chart - same bodies, different compartments, a different
          rising sign - and not merely that it renders. */}
      <ChartCase
        title="Chart · D9 navamsa, same instant"
        chart={data.chakraNavamsa}
        format="north"
        numbered={false}
      />

      {/* The degree grid as the panel actually draws it, which nothing else
          here covers: the pathway cases set `pathway` directly, and the panel
          cannot - it has one setting. That gap is why a build once drew the
          scale over bodies the packed layout had placed, a chart asserting a
          degree its bodies did not stand at. This case is the configuration a
          reader gets when they turn the setting on, so the grid and the
          placement can be checked against each other rather than separately. */}
      <ChartCase
        title="Chart · the degree grid, as the panel draws it"
        chart={data.chakra}
        format="north"
        numbered={false}
        grid
      />

      {/* D60, with the grid, as the panel draws it. The fastest division and
          the one a reader watching the chart is most likely to have open - the
          lagna crosses a part every two minutes. Its own case because the
          traversal's behaviour follows how a division scatters the bodies, and
          D60 scatters them differently from D1. */}
      <ChartCase
        title="Chart · D60 with the degree grid, as the panel draws it"
        chart={data.chakraShashtiamsa}
        format="north"
        numbered={false}
        grid
      />

      {/* The same D60 chart one crossing later - two minutes. Every sign has
          moved one house, so the same body is now in a differently shaped
          compartment. Side by side with the case above, this is what a reader
          sees change when the chart hands over. */}
      <ChartCase
        title="Chart · D60, one crossing later"
        chart={data.chakraShashtiamsaNext}
        format="north"
        numbered={false}
        grid
      />

      {/* The sky, which is the only decoration in the chart. Drawn on its own
          and again under the grid, because the two together are the densest the
          chart ever gets and the question is whether the labels still read. */}
      <ChartCase
        title="Chart · the starry sky"
        chart={data.chakra}
        format="north"
        numbered={false}
        sky
      />
      <ChartCase
        title="Chart · the sky under the degree grid"
        chart={data.chakra}
        format="north"
        numbered={false}
        sky
        grid
      />
      {/* South Indian, where the compartments are cells rather than kites: the
          sky is thinned by distance from the chart's centre, and a grid of
          cells puts labels in places a diamond does not. */}
      <ChartCase
        title="Chart · the sky, South Indian"
        chart={data.chakra}
        format="south"
        numbered={false}
        sky
      />

      {/* The handover, on a loop. It is the only motion in the chart fast enough
          to watch - the drift is a position, not a movement - and a real one is
          two hours away in D1, so the harness alternates between two charts two
          hours apart and lets it run. Everything in each compartment arrives
          through the wall it came from, clipped by the compartment's own
          outline. */}
      <HandoverCase />

      {/* The drift, stepped. An arrangement that is legal where the sign is
          entered and legal where it is left can be illegal in between, so the
          harness draws the same chart at points across the run and the
          geometric check walks all of them. This is the acceptance test for
          `docs/design/animation.md`, and the reason it is drawn rather than
          asserted in a unit test: the invariants are about rendered boxes. */}
      <For each={[0, 0.25, 0.5, 0.75, 1]}>
        {(at) => (
          <ChartCase
            title={`Chart · drift at ${Math.round(at * 100)}%`}
            chart={{
              ...data.chakra,
              lagna: { ...data.chakra.lagna, progress: at },
            }}
            format="north"
            numbered={false}
            animate
          />
        )}
      </For>

      {/* The same, in the compartment shapes that constrain it most: a crowd in
          a corner triangle has almost no room to move, which is the design
          working rather than the design failing. */}
      <For each={[0, 0.5, 1]}>
        {(at) => (
          <ChartCase
            title={`Chart · crowded drift at ${Math.round(at * 100)}%`}
            chart={{
              ...data.chakraCorner,
              lagna: { ...data.chakraCorner.lagna, progress: at },
            }}
            format="north"
            numbered={false}
            animate
          />
        )}
      </For>

      {/* ---- The pathway prototype. `docs/design/traversal.md`. -------------

          Everything below is behind `pathway` on the renderer, which nothing
          but this file sets, so the panel above is drawn by the code that
          shipped and is unaffected by any of it.

          What is being looked at: a body no longer sits where a packer put it.
          It sits at its own progress along a route across its compartment - its
          degree, in D1 - and the whole ring carries that route's contents from
          the edge the sign arrived through to the edge it will leave by as the
          lagna crosses its sign. Half the band is the degree and half is the
          crossing, because in the sky both spans are one house long. */}

      <PathwaySweep />

      {/* The traversal as motion. Five seconds a crossing against two hours in
          D1, with the degree grid on: the ticks slide down the band as the
          crossing runs, which is the arithmetic rather than an impression of
          it. */}
      <PathwayRun title="Pathway · D1, running, with the grid" chart={data.chakra} grid />

      {/* The same, in the shape that constrains it most. Seven bodies in a
          corner triangle have to lane out either side of the route, and the
          route has to keep them off a diagonal wall the whole way. */}
      <PathwayRun
        title="Pathway · seven in a corner triangle, running"
        chart={data.chakraCorner}
      />

      {/* The grid on all three compartment shapes at once, at the two ends of a
          crossing. Four kites, four corner triangles, four wall triangles: the
          question the overlay answers is whether the scale bunches into the
          point of a triangle, which is what a route drawn straight between the
          two gates would do. */}
      <For each={[0, 1]}>
        {(at) => (
          <PathwayCase
            title={`Pathway · degree grid at ${Math.round(at * 100)}%`}
            chart={data.chakra}
            progress={at}
            grid
          />
        )}
      </For>

      {/* The stations themselves, stepped. Ma at 12° of Mithuna stands further
          from the exit than Su at 3° of Simha, in both charts, at every
          progress - the degree is a station and the crossing is a translation
          the whole ring shares. */}
      <For each={[0, 0.5, 1]}>
        {(at) => (
          <PathwayCase
            title={`Pathway · D1 at ${Math.round(at * 100)}%`}
            chart={data.chakra}
            progress={at}
          />
        )}
      </For>

      {/* The three crowding cases at the two ends. Eight in a kite is the
          ceiling the whole layout is answerable to; seven in either triangle is
          the shape that runs out of room first. */}
      <For
        each={
          [
            ["eight in a kite", data.chakraCrowded],
            ["seven in a corner triangle", data.chakraCorner],
            ["seven in a wall triangle", data.chakraWall],
            ["D2 hora, eight in one", data.chakraHora],
          ] as const
        }
      >
        {([label, chart]) => (
          <For each={[0, 1]}>
            {(at) => (
              <PathwayCase
                title={`Pathway · ${label} at ${Math.round(at * 100)}%`}
                chart={chart}
                progress={at}
              />
            )}
          </For>
        )}
      </For>

      {/* D2 is the crowding ceiling, and unlike the 1962 conjunction it is not
          a once-a-century event: every chart looks like this in D2, because the
          hora maps all nine bodies into Karka and Simha alone. */}
      <ChartCase
        title="Chart · D2 hora, two compartments"
        chart={data.chakraHora}
        format="north"
        numbered={false}
      />

      <ChartCase
        title="Chart · D2 hora, South Indian"
        chart={data.chakraHora}
        format="south"
        numbered={false}
      />

      <ChartCase
        title="Chart · D30 trimsamsa, the unequal division"
        chart={data.chakraTrimsamsa}
        format="north"
        numbered={false}
      />

      {/* The same crowd in the two shapes that actually constrain the layout. A
          kite is the roomiest compartment on the chart and the case below lands
          in one, which is why it showed nothing wrong for so long: a corner
          triangle runs out of width and a wall triangle has a diagonal to cross.
          Houses rotate with the lagna, so any stellium visits all twelve over a
          day - these are the same bodies at a different hour. */}
      <ChartCase
        title="Chart · crowd in a corner triangle"
        chart={data.chakraCorner}
        format="north"
        numbered={false}
      />

      <ChartCase
        title="Chart · crowd in a wall triangle"
        chart={data.chakraWall}
        format="north"
        numbered={false}
      />

      <ChartCase
        title="Chart · crowd in a corner triangle, numbered"
        chart={data.chakraCorner}
        format="north"
        numbered={true}
      />

      {/* The ceiling: eight bodies in one sign, found by scanning two centuries
          rather than picked by hand. Eight is the most there can ever be - the
          seven grahas can all share a sign and exactly one node can join them,
          because Rahu and Ketu are opposite by construction. This is February
          1962, and it is the case every layout rule here is answerable to. */}
      <ChartCase
        title="Chart · eight in one sign (Feb 1962)"
        chart={data.chakraCrowded}
        format="north"
        numbered={false}
      />

      {/* The same crowding in the two formats whose compartments are a fixed
          grid rather than a set of kites and triangles, so the row that has to
          be squeezed is a different shape. */}
      <ChartCase
        title="Chart · eight in one sign, South Indian"
        chart={data.chakraCrowded}
        format="south"
        numbered={false}
      />

      <ChartCase
        title="Chart · reduced precision (1650)"
        chart={data.chakraMoshier}
        format="north"
        numbered={false}
      />

      <ChartCase
        title="Chart · error state"
        chart={undefined}
        format="north"
        numbered={false}
        error={{
          code: "ENGINE",
          message: "the ephemeris could not be read",
        }}
      />

      <section class="preview__case preview__case--wide">
        <h2 class="preview__title">Phase sequence · 14px</h2>
        <div class="preview__strip">
          <For each={[0, 0.12, 0.25, 0.38, 0.5, 0.62, 0.75, 0.88, 1]}>
            {(value) => (
              <span class="preview__swatch">
                <PhaseGlyph illumination={value} waxing size={14} />
                <span>{Math.round(value * 100)}</span>
              </span>
            )}
          </For>
          <For each={[0.88, 0.75, 0.62, 0.5, 0.38, 0.25, 0.12]}>
            {(value) => (
              <span class="preview__swatch">
                <PhaseGlyph illumination={value} waxing={false} size={14} />
                <span>{Math.round(value * 100)}</span>
              </span>
            )}
          </For>
        </div>
      </section>

      <section class="preview__case preview__case--wide">
        <h2 class="preview__title">Navagraha glyphs · 20px</h2>
        <div class="preview__strip">
          <For each={data.grahas}>
            {(graha) => (
              <span class="preview__swatch">
                <GrahaGlyph info={graha} size={20} />
                <span>{graha.name}</span>
              </span>
            )}
          </For>
        </div>
      </section>
    </div>
  );
}
