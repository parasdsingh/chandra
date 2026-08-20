/**
 * Visual harness. Development only.
 *
 * Renders the panel's presentational components against real almanac output
 * (`fixture.json`, produced by `cargo run -p chandra --example preview_data`),
 * so layout, spacing, glyphs and degraded states can be inspected in a browser
 * without a menu bar.
 *
 * It touches no IPC and is reachable only at `?preview` in a dev build, so it
 * cannot appear in the shipped app.
 */

import type { JSX } from "solid-js";
import { For } from "solid-js";

import fixture from "./fixture.json";
import { DayDetail } from "../components/DayDetail";
import { Header } from "../components/Header";
import { MonthGrid } from "../components/MonthGrid";
import { MonthPicker } from "../components/MonthPicker";
import { PhaseGlyph } from "../components/PhaseGlyph";
import { GrahaGlyph } from "../components/GrahaGlyph";
import { SettingsView } from "../components/Settings";
import type {
  Bootstrap,
  DayDetail as Detail,
  GrahaInfo,
  GrahaMonth,
  MoonMonth,
  Snapshot,
} from "../ipc/types";
import { buildGrid } from "../lib/calendar";
import type { FormatContext } from "../lib/format";

const data = fixture as unknown as {
  timeZone: string;
  grahas: GrahaInfo[];
  moonMonth: MoonMonth;
  moonDay: Detail;
  moonDayNoRise: Detail;
  grahaMonth: GrahaMonth;
  grahaDay: Detail;
  moshierDay: Detail;
  snapshot: Snapshot;
};

/** A bootstrap payload built from the same fixture, for the settings preview. */
const settingsBoot: Bootstrap = {
  settings: {
    schema_version: 1,
    launch_at_login: false,
    time_format: "hour24",
    location: {
      mode: "manual",
      place: {
        label: "Bengaluru",
        zone: "Asia/Kolkata",
        latitude: 12.9716,
        longitude: 77.5946,
        elevation: 920,
      },
    },
    sidereal: { ayanamsa: "lahiri", node_type: "true" },
    tray: { subjects: ["mangala", "shani"], colour_mode: false },
  },
  location: {
    label: "Bengaluru",
    zone: "Asia/Kolkata",
    latitude: 12.9716,
    longitude: 77.5946,
    elevation: 920,
    provenance: "manual",
  },
  subjects: ["mangala", "shani"],
  library_version: "2.10.03",
  ayanamsas: [
    { key: "lahiri", label: "Lahiri (Chitrapaksha)" },
    { key: "raman", label: "Raman" },
    { key: "krishnamurti", label: "Krishnamurti (KP)" },
  ],
  node_types: [
    { key: "true", label: "True node" },
    { key: "mean", label: "Mean node" },
  ],
  grahas: data.grahas,
};

const context: FormatContext = {
  timeZone: data.timeZone,
  timeFormat: "hour24",
};

function Case(props: { title: string; children: JSX.Element }): JSX.Element {
  return (
    <section class="preview__case">
      <h2 class="preview__title">{props.title}</h2>
      <div class="panel-frame">{props.children}</div>
    </section>
  );
}

export function Preview(): JSX.Element {
  const moonGrid = buildGrid(2026, 8, data.moonMonth.leading_blanks, 0);
  const grahaGrid = buildGrid(2025, 2, data.grahaMonth.leading_blanks, 0);
  const chandra = data.grahas.find((graha) => graha.key === "chandra");
  const mangala = data.grahas.find((graha) => graha.key === "mangala");

  // A function, not a shared const: a JSX expression assigned to a variable is
  // created once, so reusing it across cases moves one DOM node between them
  // instead of rendering three.
  const moonHeader = () => (
    <Header
      subject="chandra"
      subjectName="Chandra"
      info={chandra}
      snapshot={data.snapshot}
      southern={false}
      year={2026}
      month={8}
      pickerOpen={false}
      onTogglePicker={() => {}}
      onStep={() => {}}
      onSettings={() => {}}
    />
  );

  return (
    <div class="preview">
      <Case title="Moon · collapsed">
        <div class="panel">
          {moonHeader()}
          <MonthGrid
            grid={moonGrid}
            firstWeekday={0}
            month={data.moonMonth}
            kind="moon"
            selected={null}
            today={{ year: 2026, month: 8, day: 20 }}
            southern={false}
            direction={0}
            onSelect={() => {}}
          />
        </div>
      </Case>

      <Case title="Moon · day expanded">
        <div class="panel">
          {moonHeader()}
          <MonthGrid
            grid={moonGrid}
            firstWeekday={0}
            month={data.moonMonth}
            kind="moon"
            selected={{ year: 2026, month: 8, day: 20 }}
            today={{ year: 2026, month: 8, day: 20 }}
            southern={false}
            direction={0}
            onSelect={() => {}}
          />
          <div class="panel__divider" />
          <DayDetail
            detail={data.moonDay}
            events={[]}
            grahaName="Chandra"
            context={context}
            isToday
            error={undefined}
          />
        </div>
      </Case>

      <Case title="Moon · a day with no moonrise">
        <div class="panel">
          <DayDetail
            detail={data.moonDayNoRise}
            events={[]}
            grahaName="Chandra"
            context={context}
            isToday={false}
            error={undefined}
          />
        </div>
      </Case>

      <Case title="Moon · reduced precision (1650)">
        <div class="panel">
          <DayDetail
            detail={data.moshierDay}
            events={[]}
            grahaName="Chandra"
            context={context}
            isToday={false}
            error={undefined}
          />
        </div>
      </Case>

      <Case title="Mangala · retrograde month, station day selected">
        <div class="panel">
          <Header
            subject="mangala"
            subjectName="Mangala"
            info={mangala}
            snapshot={undefined}
            southern={false}
            year={2025}
            month={2}
            pickerOpen={false}
            onTogglePicker={() => {}}
            onStep={() => {}}
            onSettings={() => {}}
          />
          <MonthGrid
            grid={grahaGrid}
            firstWeekday={0}
            month={data.grahaMonth}
            kind="graha"
            selected={{ year: 2025, month: 2, day: 24 }}
            today={{ year: 2025, month: 2, day: 10 }}
            southern={false}
            direction={0}
            onSelect={() => {}}
          />
          <div class="panel__divider" />
          <DayDetail
            detail={data.grahaDay}
            events={data.grahaMonth.events.filter(
              (event) => event.date.day === 24,
            )}
            grahaName="Mangala"
            context={context}
            isToday={false}
            error={undefined}
          />
        </div>
      </Case>

      <Case title="Month picker">
        <div class="panel">
          {moonHeader()}
          <MonthPicker
            year={2026}
            month={8}
            today={{ year: 2026, month: 8 }}
            onPick={() => {}}
            onStepYear={() => {}}
          />
        </div>
      </Case>

      <Case title="Error state">
        <div class="panel">
          <DayDetail
            detail={undefined}
            events={[]}
            grahaName="Chandra"
            context={context}
            isToday={false}
            error={{
              code: "DATE_OUT_OF_RANGE",
              message: "1650-08-20 is outside the range Chandra has data for",
            }}
          />
        </div>
      </Case>

      <section class="preview__case preview__case--wide">
        <h2 class="preview__title">Phase sequence · in-panel glyph at 14px</h2>
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

      <For each={["location", "astrology", "grahas", "about"] as const}>
        {(tab) => (
          <section class="preview__case">
            <h2 class="preview__title">Settings · {tab}</h2>
            <div class="preview__settings">
              <SettingsView
                boot={settingsBoot}
                apply={() => {}}
                busy={false}
                failure={undefined}
                initialTab={tab}
              />
            </div>
          </section>
        )}
      </For>

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
