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
import { batch, createEffect, createSignal, For, onCleanup, Show } from "solid-js";

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
  | "size"
  | "about";

export const SECTION_TITLES: Record<SettingsSection, string> = {
  root: "Settings",
  calendar: "Calendar",
  location: "Location",
  astrology: "Astrology",
  menubar: "Menu bar",
  size: "Size",
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
      <Show when={props.section === "size"}>
        <Size boot={props.boot} apply={props.apply} />
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
    { id: "size", value: () => sizeLabel(settings().appearance.scale) },
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

/**
 * The panel's sizes, as multipliers.
 *
 * Steps rather than a slider: the panel is a fixed composition and only a few
 * sizes of it look composed. The names say what the user gets rather than what
 * the number is - nobody wants "1.15".
 */
const SIZES: { scale: number; label: string }[] = [
  { scale: 0.85, label: "Compact" },
  { scale: 1.0, label: "Default" },
  { scale: 1.15, label: "Large" },
  { scale: 1.3, label: "Larger" },
];

function sizeLabel(scale: number): string {
  const nearest = SIZES.reduce((best, size) =>
    Math.abs(size.scale - scale) < Math.abs(best.scale - scale) ? size : best,
  );
  return nearest.label;
}

/**
 * Panel size.
 *
 * Everything moves together - the window, the grid, the type, the marks - so a
 * bigger panel is the same design drawn larger rather than a different one. The
 * calendar keeps its shape at every step, which is the whole reason this is one
 * multiplier and not a type-size setting.
 */
function Size(props: SectionProps): JSX.Element {
  const settings = () => props.boot.settings;

  return (
    <div class="settings__section">
      <p class="settings__hint">
        Changes the whole panel — the window, the calendar, the type and the
        marks together.
      </p>
      <div role="radiogroup" aria-label="Panel size">
        <For each={SIZES}>
          {(size) => (
            <Choice
              label={size.label}
              selected={
                sizeLabel(settings().appearance.scale) === size.label
              }
              onSelect={() =>
                props.apply({
                  ...settings(),
                  appearance: { scale: size.scale },
                })
              }
            />
          )}
        </For>
      </div>
    </div>
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
      {/* Says the scope out loud. The section is reached from whichever
          subject's panel happens to be open, which made a setting that has
          always been one calendar-wide choice read as that subject's own. */}
      <p class="settings__hint">
        Used by every calendar — the Moon and each graha alike.
      </p>
      <p class="settings__hint">
        A lunar month runs between syzygies, not between calendar dates, and is
        named from where the Sun stands at that moment.
      </p>
      <ChoiceGroup label="Month system">
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
      </ChoiceGroup>
    </div>
  );
}

/** Quiet time after the last keystroke before a search is sent. */
const SEARCH_SETTLE_MS = 180;

function Location(props: SectionProps): JSX.Element {
  const [query, setQuery] = createSignal("");
  const [results, setResults] = createSignal<City[]>([]);
  const [searchFailed, setSearchFailed] = createSignal(false);
  const [locating, setLocating] = createSignal(false);
  const settings = () => props.boot.settings;

  /**
   * One search per pause, and only the newest answer counts.
   *
   * A search per keystroke put several in flight at once, and a slower earlier
   * one landing last replaced the list with matches for a prefix the user had
   * already typed past. A rejection became an unhandled promise rejection and
   * left the pane saying no city matched, which is a statement about the data
   * for what was actually a failure to ask.
   */
  createEffect(() => {
    const text = query().trim();
    if (text.length < 2) {
      batch(() => {
        setResults([]);
        setSearchFailed(false);
      });
      return;
    }

    let current = true;
    const timer = window.setTimeout(() => {
      void ipc
        .searchCities(text, 6)
        .then((cities) => {
          if (!current) return;
          batch(() => {
            setResults(cities);
            setSearchFailed(false);
          });
        })
        .catch(() => {
          if (!current) return;
          batch(() => {
            setResults([]);
            setSearchFailed(true);
          });
        });
    }, SEARCH_SETTLE_MS);

    onCleanup(() => {
      current = false;
      window.clearTimeout(timer);
    });
  });

  function choose(city: City) {
    props.apply({
      ...settings(),
      location: {
        ...settings().location,
        mode: "manual",
        place: {
          label: city.city,
          zone: city.zone,
          latitude: city.latitude,
          longitude: city.longitude,
          // The table carries no elevation. The user's own correction lives
          // beside the place and still applies, so it is not copied here.
          elevation: 0,
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
            <p class="settings__empty">
              {searchFailed()
                ? "City search is unavailable."
                : `No city matches \u201c${query().trim()}\u201d.`}
            </p>
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
              // The observer's own correction, not part of the place. Writing a
              // whole place around it made a timezone-derived location report
              // itself as having come from this Mac.
              props.apply({
                ...settings(),
                location: {
                  ...settings().location,
                  elevation: Number(event.currentTarget.value),
                },
              });
            }}
          />
        </div>

        <div class="settings__actions">
          {/* Not offered while a chosen location is in force. A manual override
              is authoritative and is never overridden (D-007), so the button
              granted permission, took CoreLocation's answer and dropped it
              without saying anything. */}
          <Show when={settings().location.mode !== "manual"}>
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
          </Show>
          {/* Offered whenever a stored place is standing in for the timezone,
              which includes a cached device fix. Showing it only in manual mode
              left an automatic place with no way back to the chain. */}
          <Show
            when={
              settings().location.mode === "manual" ||
              settings().location.place !== null
            }
          >
            <button
              class="settings__button"
              onClick={() =>
                props.apply({
                  ...settings(),
                  location: {
                    ...settings().location,
                    mode: "automatic",
                    place: null,
                  },
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
      <ChoiceGroup label="Ayanamsa">
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
      </ChoiceGroup>

      <p class="settings__group">Rahu and Ketu</p>
      <ChoiceGroup label="Rahu and Ketu">
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
      </ChoiceGroup>
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

      <Toggle
        label="Coloured icons"
        on={settings().tray.colour_mode}
        onToggle={() =>
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

/**
 * A set of mutually exclusive choices.
 *
 * The element exists for the role. A radio outside a radiogroup announces itself
 * as one of one, so the set of ayanamsas read as eleven unrelated controls each
 * claiming to be the only option it had.
 */
function ChoiceGroup(props: { label: string; children: JSX.Element }): JSX.Element {
  return (
    <div class="settings__choices" role="radiogroup" aria-label={props.label}>
      {props.children}
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

/**
 * A row that turns one thing on or off.
 *
 * Drawn exactly like a `Choice`, but it is not a radio: a radio is one of
 * several and cannot be switched off again once chosen, which is the opposite of
 * what this does. `switch` is the role for a control with two states.
 */
function Toggle(props: {
  label: string;
  on: boolean;
  onToggle: () => void;
}): JSX.Element {
  return (
    <button
      class="settings__choice"
      classList={{ "is-selected": props.on }}
      role="switch"
      aria-checked={props.on}
      onClick={props.onToggle}
    >
      <span>{props.label}</span>
      <Show when={props.on}>
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
