/**
 * Visual harness. Development only.
 *
 * Renders the panel's views against real almanac output (`fixture.json`, from
 * `make preview`), so layout, spacing, glyphs and degraded states can be
 * inspected without a menu bar. It touches no IPC and is reachable only at
 * `?preview` in a dev build.
 */

import type { JSX } from "solid-js";
import { createSignal, For } from "solid-js";

import fixture from "./fixture.json";
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
  chakraHora: Chakra;
  chakraTrimsamsa: Chakra;
  chakraCorner: Chakra;
  chakraWall: Chakra;
  chakraMoshier: Chakra;
};

const context: FormatContext = { timeZone: data.timeZone };

const boot: Bootstrap = {
  settings: {
    schema_version: 11,
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
    chart: { format: "north", vargas: ["d1", "d9"], numbered: false },
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
    { key: "d1", label: "D1 · Rashi", name: "Rashi", division: 1, reads: "the physique" },
    { key: "d2", label: "D2 · Hora", name: "Hora", division: 2, reads: "wealth" },
    { key: "d9", label: "D9 · Navamsa", name: "Navamsa", division: 9, reads: "the spouse" },
    { key: "d30", label: "D30 · Trimsamsa", name: "Trimsamsa", division: 30, reads: "evils" },
  ] as const,
};

/**
 * One chart, in the panel it is drawn in.
 *
 * The header comes with it rather than the chart alone: the title is built from
 * the reading - the rising sign, not the feature's name - so a chart whose
 * header says the wrong thing is only visible when both are on screen together.
 */
function ChartCase(props: {
  title: string;
  chart: Chakra | undefined;
  format: ChartFormat;
  numbered: boolean;
  error?: { code: string; message: string };
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
        />
      </div>
    </Case>
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
