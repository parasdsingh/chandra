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
  lunarDay: Detail;
  moonDayNoRise: Detail;
  grahaMonth: GrahaMonth;
  grahaDay: Detail;
  moshierDay: Detail;
  snapshot: Snapshot;
};

const context: FormatContext = { timeZone: data.timeZone };

const boot: Bootstrap = {
  settings: {
    schema_version: 2,
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
    calendar: { month_system: "amanta" },
    tray: { subjects: ["mangala", "shani"], colour_mode: false },
  appearance: { scale: 1 },
  },
  location: {
    label: "Bengaluru",
    zone: "Asia/Kolkata",
    latitude: 12.9716,
    longitude: 77.5946,
    elevation: 920,
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
};

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
              active
              onSelect={() => {}}
            />
          </div>
        </div>
      </Case>

      <Case title="Moon · day view">
        <Header
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
        />
        <div class="region">
          <DayDetail
            detail={data.moonDay}
            events={[]}
            context={context}
            isToday
            error={undefined}
          />
        </div>
      </Case>

      <Case title="Moon · day view, lunar month">
        <div class="region">
          <DayDetail
            detail={data.lunarDay}
            events={[]}
            context={context}
            isToday={false}
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
            error={undefined}
          />
        </div>
      </Case>

      <Case title="Mangala · retrograde month">
        <Header
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
            events={data.grahaMonth.events.filter((event) => event.date.day === 24)}
            context={context}
            isToday={false}
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
            error={{
              code: "DATE_OUT_OF_RANGE",
              message: "1650-08-20 is outside the range Chandra has data for",
            }}
          />
        </div>
      </Case>

      <For each={["root", "calendar", "location", "astrology", "menubar",
            "size", "about"] as const}>
        {(id) => (
          <Case title={`Settings · ${id}`}>
            <Header
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
