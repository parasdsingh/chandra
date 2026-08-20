/**
 * Settings, inside the panel.
 *
 * There is no separate settings window. The panel is 320px wide with a 264px
 * region, which will not hold five tabbed panes side by side, so settings are a
 * drill-down: a short list of sections, each opening in the same region with the
 * header carrying the way back. That is also fewer surfaces to learn - the panel
 * is the only place the app has.
 *
 * Every change applies immediately. Nothing here needs confirming: each setting
 * is reversible and its effect is visible in the calendar behind it.
 */

import type { JSX } from "solid-js";
import { createEffect, createSignal, For, Show } from "solid-js";

import * as ipc from "../ipc";
import type {
  Bootstrap,
  City,
  GrahaKey,
  MonthSystem,
  Settings,
} from "../ipc/types";
import { GrahaGlyph } from "./GrahaGlyph";

export type SettingsSection =
  | "root"
  | "calendar"
  | "location"
  | "astrology"
  | "menubar"
  | "about";

export const SECTION_TITLES: Record<SettingsSection, string> = {
  root: "Settings",
  calendar: "Calendar",
  location: "Location",
  astrology: "Astrology",
  menubar: "Menu bar",
  about: "About",
};

interface Props {
  boot: Bootstrap;
  section: SettingsSection;
  onOpen: (section: SettingsSection) => void;
  apply: (next: Settings) => void;
}

export function SettingsView(props: Props): JSX.Element {
  return (
    <div class="settings">
      <Show when={props.section === "root"}>
        <Root boot={props.boot} onOpen={props.onOpen} />
      </Show>
      <Show when={props.section === "calendar"}>
        <Calendar boot={props.boot} apply={props.apply} />
      </Show>
      <Show when={props.section === "location"}>
        <Location boot={props.boot} apply={props.apply} />
      </Show>
      <Show when={props.section === "astrology"}>
        <Astrology boot={props.boot} apply={props.apply} />
      </Show>
      <Show when={props.section === "menubar"}>
        <MenuBar boot={props.boot} apply={props.apply} />
      </Show>
      <Show when={props.section === "about"}>
        <About boot={props.boot} />
      </Show>
    </div>
  );
}

/** Section list. Each row states where it leads and what it is set to now. */
function Root(props: {
  boot: Bootstrap;
  onOpen: (section: SettingsSection) => void;
}): JSX.Element {
  const settings = () => props.boot.settings;

  const systemLabel = () =>
    props.boot.month_systems.find(
      (choice) => choice.key === settings().calendar.month_system,
    )?.label ?? "";

  const ayanamsaLabel = () =>
    props.boot.ayanamsas.find(
      (choice) => choice.key === settings().sidereal.ayanamsa,
    )?.label ?? "";

  const trayCount = () => settings().tray.subjects.length;

  const rows: { id: SettingsSection; value: () => string }[] = [
    { id: "calendar", value: () => shortSystem(settings().calendar.month_system) },
    { id: "location", value: () => props.boot.location.label },
    { id: "astrology", value: () => ayanamsaLabel().split(" ")[0] ?? "" },
    {
      id: "menubar",
      value: () =>
        trayCount() === 0 ? "Moon only" : `Moon + ${trayCount()}`,
    },
    { id: "about", value: () => "" },
  ];

  return (
    <nav class="settings__list">
      <For each={rows}>
        {(row) => (
          <button class="settings__nav" onClick={() => props.onOpen(row.id)}>
            <span class="settings__nav-title">{SECTION_TITLES[row.id]}</span>
            <span class="settings__nav-value">{row.value()}</span>
            <Chevron />
          </button>
        )}
      </For>
      <p class="settings__footnote">
        {systemLabel()} · {props.boot.location.zone}
      </p>
    </nav>
  );
}

function shortSystem(system: MonthSystem): string {
  switch (system) {
    case "solar":
      return "Solar";
    case "amanta":
      return "Amanta";
    case "purnimanta":
      return "Purnimanta";
  }
}

interface SectionProps {
  boot: Bootstrap;
  apply: (next: Settings) => void;
}

function Calendar(props: SectionProps): JSX.Element {
  const settings = () => props.boot.settings;

  return (
    <div class="settings__section">
      <p class="settings__hint">
        A lunar month runs between syzygies, not between calendar dates, and is
        named from where the Sun stands at that moment.
      </p>
      <For each={props.boot.month_systems}>
        {(choice) => (
          <Choice
            label={choice.label}
            selected={settings().calendar.month_system === choice.key}
            onSelect={() =>
              props.apply({
                ...settings(),
                calendar: { month_system: choice.key as MonthSystem },
              })
            }
          />
        )}
      </For>
    </div>
  );
}

function Location(props: SectionProps): JSX.Element {
  const [query, setQuery] = createSignal("");
  const [results, setResults] = createSignal<City[]>([]);
  const [locating, setLocating] = createSignal(false);
  const settings = () => props.boot.settings;

  createEffect(() => {
    const text = query();
    if (text.trim().length < 2) {
      setResults([]);
      return;
    }
    void ipc.searchCities(text, 6).then(setResults);
  });

  function choose(city: City) {
    props.apply({
      ...settings(),
      location: {
        mode: "manual",
        place: {
          label: city.city,
          zone: city.zone,
          latitude: city.latitude,
          longitude: city.longitude,
          elevation: settings().location.place?.elevation ?? 0,
        },
      },
    });
    setQuery("");
    setResults([]);
  }

  return (
    <div class="settings__section">
      <div class="settings__current">
        <span class="settings__current-place">{props.boot.location.label}</span>
        <span class="settings__hint">
          {props.boot.location.latitude.toFixed(3)},{" "}
          {props.boot.location.longitude.toFixed(3)} ·{" "}
          {props.boot.location.elevation.toFixed(0)} m ·{" "}
          {provenanceLabel(props.boot.location.provenance)}
        </span>
      </div>

      <input
        class="settings__search"
        type="search"
        placeholder="Search for a city"
        value={query()}
        onInput={(event) => setQuery(event.currentTarget.value)}
      />

      <Show when={query().trim().length >= 2}>
        <Show
          when={results().length > 0}
          fallback={
            <p class="settings__empty">No city matches “{query().trim()}”.</p>
          }
        >
          <ul class="settings__results">
            <For each={results()}>
              {(city) => (
                <li>
                  <button class="settings__result" onClick={() => choose(city)}>
                    <span>{city.city}</span>
                    <span class="settings__hint">{city.country}</span>
                  </button>
                </li>
              )}
            </For>
          </ul>
        </Show>
      </Show>

      <Show when={query().trim().length < 2}>
        <div class="settings__row">
          <span class="settings__label">Elevation</span>
          <input
            class="settings__number"
            type="number"
            step="10"
            value={props.boot.location.elevation}
            onChange={(event) => {
              const place = settings().location.place;
              const elevation = Number(event.currentTarget.value);
              props.apply({
                ...settings(),
                location: {
                  mode: settings().location.mode,
                  place: place
                    ? { ...place, elevation }
                    : {
                        label: props.boot.location.label,
                        zone: props.boot.location.zone,
                        latitude: props.boot.location.latitude,
                        longitude: props.boot.location.longitude,
                        elevation,
                      },
                },
              });
            }}
          />
        </div>

        <div class="settings__actions">
          <button
            class="settings__button"
            disabled={locating()}
            onClick={() => {
              setLocating(true);
              void ipc.requestDeviceLocation().finally(() => setLocating(false));
            }}
          >
            {locating() ? "Asking macOS…" : "Use this Mac"}
          </button>
          <Show when={settings().location.mode === "manual"}>
            <button
              class="settings__button"
              onClick={() =>
                props.apply({
                  ...settings(),
                  location: { mode: "automatic", place: null },
                })
              }
            >
              Use my timezone
            </button>
          </Show>
        </div>
      </Show>
    </div>
  );
}

function provenanceLabel(provenance: Bootstrap["location"]["provenance"]): string {
  switch (provenance) {
    case "manual":
      return "chosen";
    case "core_location":
      return "from this Mac";
    case "time_zone":
      return "from your timezone";
  }
}

function Astrology(props: SectionProps): JSX.Element {
  const settings = () => props.boot.settings;

  return (
    <div class="settings__section settings__section--scroll">
      <p class="settings__group">Ayanamsa</p>
      <For each={props.boot.ayanamsas}>
        {(choice) => (
          <Choice
            label={choice.label}
            selected={settings().sidereal.ayanamsa === choice.key}
            onSelect={() =>
              props.apply({
                ...settings(),
                sidereal: { ...settings().sidereal, ayanamsa: choice.key },
              })
            }
          />
        )}
      </For>

      <p class="settings__group">Rahu and Ketu</p>
      <For each={props.boot.node_types}>
        {(choice) => (
          <Choice
            label={choice.label}
            selected={settings().sidereal.node_type === choice.key}
            onSelect={() =>
              props.apply({
                ...settings(),
                sidereal: { ...settings().sidereal, node_type: choice.key },
              })
            }
          />
        )}
      </For>
    </div>
  );
}

function MenuBar(props: SectionProps): JSX.Element {
  const settings = () => props.boot.settings;

  function toggle(graha: GrahaKey, on: boolean) {
    const subjects = new Set(settings().tray.subjects);
    if (on) subjects.add(graha);
    else subjects.delete(graha);
    props.apply({
      ...settings(),
      tray: { ...settings().tray, subjects: [...subjects] },
    });
  }

  return (
    <div class="settings__section settings__section--scroll">
      <For each={props.boot.grahas}>
        {(graha) => {
          const permanent = graha.key === "chandra";
          const on = () => permanent || settings().tray.subjects.includes(graha.key);
          return (
            <button
              class="settings__toggle"
              classList={{ "is-on": on(), "is-locked": permanent }}
              disabled={permanent}
              aria-pressed={on()}
              onClick={() => toggle(graha.key, !on())}
            >
              <GrahaGlyph info={graha} size={18} />
              <span class="settings__toggle-name">{graha.name}</span>
              <span class="settings__hint">{graha.english}</span>
              <span class="settings__switch" aria-hidden="true" />
            </button>
          );
        }}
      </For>

      <p class="settings__hint settings__hint--foot">
        {/* The permanent moon item is Chandra's item; a second one would put two
            moons in the menu bar. */}
        The moon is always shown.
      </p>

      <Choice
        label="Coloured icons"
        selected={settings().tray.colour_mode}
        onSelect={() =>
          props.apply({
            ...settings(),
            tray: {
              ...settings().tray,
              colour_mode: !settings().tray.colour_mode,
            },
          })
        }
      />
      <p class="settings__hint">
        Off by default: template icons follow the menu bar and stay visible in
        both appearances.
      </p>
    </div>
  );
}

function About(props: { boot: Bootstrap }): JSX.Element {
  return (
    <div class="settings__section">
      <div class="settings__row">
        <span class="settings__label">Chandra</span>
        <span class="settings__value">0.1.0</span>
      </div>
      <div class="settings__row">
        <span class="settings__label">Ephemeris</span>
        <span class="settings__value">
          Swiss Ephemeris {props.boot.library_version}
        </span>
      </div>
      <div class="settings__row">
        <span class="settings__label">Full precision</span>
        <span class="settings__value">1800–2399</span>
      </div>
      <div class="settings__row">
        <span class="settings__label">Licence</span>
        <span class="settings__value">AGPL-3.0</span>
      </div>
      <p class="settings__hint settings__hint--foot">
        Positions are computed on this Mac. Nothing is sent anywhere.
      </p>
    </div>
  );
}

/** A single-choice row. Selection is a mark, not a control that could be half-set. */
function Choice(props: {
  label: string;
  selected: boolean;
  onSelect: () => void;
}): JSX.Element {
  return (
    <button
      class="settings__choice"
      classList={{ "is-selected": props.selected }}
      role="radio"
      aria-checked={props.selected}
      onClick={props.onSelect}
    >
      <span>{props.label}</span>
      <Show when={props.selected}>
        <Tick />
      </Show>
    </button>
  );
}

function Tick(): JSX.Element {
  return (
    <svg width="14" height="14" viewBox="0 0 14 14" aria-hidden="true">
      <path
        d="M 2.5 7.5 L 5.5 10.5 L 11.5 3.5"
        fill="none"
        stroke="currentColor"
        stroke-width="1.6"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
  );
}

function Chevron(): JSX.Element {
  return (
    <svg width="14" height="14" viewBox="0 0 14 14" aria-hidden="true">
      <path
        d="M 5 2.5 L 9.5 7 L 5 11.5"
        fill="none"
        stroke="currentColor"
        stroke-width="1.4"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
  );
}
