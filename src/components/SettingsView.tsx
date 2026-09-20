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
import {
  batch,
  createEffect,
  createSignal,
  For,
  on,
  onCleanup,
  Show,
} from "solid-js";

import * as ipc from "../ipc";
import type {
  Bootstrap,
  City,
  GrahaKey,
  ChartFormat,
  IngressMode,
  MonthSystem,
  Resolved,
  Settings,
  VargaKey,
} from "../ipc/types";
import { GrahaGlyph } from "./GrahaGlyph";

export type SettingsSection =
  | "root"
  | "calendar"
  | "panchanga"
  | "chart"
  | "motion"
  | "grid"
  | "sky"
  | "advanced"
  | "ingress"
  | "compartments"
  | "location"
  | "astrology"
  | "menubar"
  | "size"
  | "about";

export const SECTION_TITLES: Record<SettingsSection, string> = {
  root: "Settings",
  calendar: "Calendar",
  panchanga: "Panchanga",
  chart: "Lagna Kundali",
  motion: "Motion",
  grid: "Degree grid",
  sky: "Starry sky",
  advanced: "Advanced",
  ingress: "Ingress labels",
  compartments: "Compartments",
  location: "Location",
  astrology: "Astrology",
  // Not "Menu bar". Two sections put items in the menu bar now - this one and
  // the chart's - so naming one of them after the menu bar says nothing about
  // which. This section is the nine calendars; the other is the divisions.
  menubar: "Calendars",
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
      <Show when={props.section === "panchanga"}>
        <Panchanga boot={props.boot} apply={props.apply} />
      </Show>
      <Show when={props.section === "chart"}>
        <Chart boot={props.boot} apply={props.apply} />
      </Show>
      <Show when={props.section === "motion"}>
        <Motion boot={props.boot} apply={props.apply} />
      </Show>
      <Show when={props.section === "grid"}>
        <Grid boot={props.boot} apply={props.apply} />
      </Show>
      <Show when={props.section === "sky"}>
        <Sky boot={props.boot} apply={props.apply} />
      </Show>

      <Show when={props.section === "advanced"}>
        <Advanced boot={props.boot} onOpen={props.onOpen} />
      </Show>
      <Show when={props.section === "ingress"}>
        <Ingress boot={props.boot} apply={props.apply} />
      </Show>
      <Show when={props.section === "compartments"}>
        <Compartments boot={props.boot} apply={props.apply} />
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

  const trayCount = () => settings().tray.subjects.length;

  const rows: { id: SettingsSection; value: () => string }[] = [
    {
      id: "calendar",
      value: () => shortSystem(settings().calendar.month_system),
    },
    { id: "location", value: () => props.boot.location.label },
    {
      id: "chart",
      // How many charts are in the menu bar, not the format they are drawn in.
      // Two people with the same format and different divisions are reading
      // different charts; the reverse is the same chart drawn two ways.
      value: () => {
        const count = Math.max(1, settings().chart.vargas.length);
        return count === 1 ? "D1 only" : `${count} divisions`;
      },
    },
    {
      id: "menubar",
      // How many are on, not what they are. The section is named `Calendars`, so
      // the row does not have to repeat the word - and `None` is a real state
      // here since D-030, which is why it is not `Chart only`: that named the
      // other section's business.
      value: () =>
        trayCount() === 0 ? "None" : `${trayCount()} of 9`,
    },
    { id: "size", value: () => sizeLabel(settings().appearance.scale) },
    { id: "advanced", value: () => "" },
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
  { scale: 0.8, label: "Compact" },
  { scale: 1.0, label: "Default" },
  { scale: 1.2, label: "Large" },
  { scale: 1.4, label: "Larger" },
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
              selected={sizeLabel(settings().appearance.scale) === size.label}
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
                  calendar: {
                    ...settings().calendar,
                    month_system: choice.key as MonthSystem,
                  },
                })
              }
            />
          )}
        </For>
      </ChoiceGroup>
    </div>
  );
}

const INGRESS_CHOICES: { key: IngressMode; label: string; short: string }[] = [
  { key: "off", label: "None", short: "Off" },
  // Western three-letter forms, because the Sanskrit names cannot be
  // abbreviated and stay distinct - Vrishabha and Vrishchika are both `Vri`.
  { key: "rashi", label: "Rashi — Ari, Tau, Gem", short: "Rashi" },
  // Two parts where the name has two: six nakshatras begin Purva or Uttara and
  // three of each share what follows.
  { key: "nakshatra", label: "Nakshatra — Ashw, P.Ash", short: "Nakshatra" },
];

/** Quiet time after the last keystroke before a search is sent. */
const SEARCH_SETTLE_MS = 180;

/**
 * A city search that reports what was picked.
 *
 * Shared by the settings pane and the first-run gate. Extracted rather than
 * copied: the debounce, the stale-answer guard and the difference between "no
 * city matches" and "the search could not be asked" are the substance of it, and
 * a second copy would be a second place for them to be got wrong.
 */
export function CitySearch(props: {
  onPick: (city: City) => void;
  /** Told whether a result list is showing, so a caller can fold away whatever
   *  sits beneath it rather than be pushed off the pane. */
  onSearchingChange?: (searching: boolean) => void;
}): JSX.Element {
  const [query, setQuery] = createSignal("");
  const [results, setResults] = createSignal<City[]>([]);
  const [searchFailed, setSearchFailed] = createSignal(false);

  const searching = () => query().trim().length >= 2;
  createEffect(() => props.onSearchingChange?.(searching()));

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

  function pick(city: City) {
    setQuery("");
    setResults([]);
    props.onPick(city);
  }

  return (
    <>
      <input
        class="settings__search"
        type="search"
        placeholder="Search for a city"
        value={query()}
        onInput={(event) => setQuery(event.currentTarget.value)}
      />

      <Show when={searching()}>
        <Show
          when={results().length > 0}
          fallback={
            <p class="settings__empty">
              {searchFailed()
                ? "City search is unavailable."
                : `No city matches “${query().trim()}”.`}
            </p>
          }
        >
          <ul class="settings__results">
            <For each={results()}>
              {(city) => (
                <li>
                  <button class="settings__result" onClick={() => pick(city)}>
                    <span>{city.city}</span>
                    {/* The region first, then the country. 1309 of the 34,129
                        city names are not unique, so `Springfield · United
                        States` eight times over is a list with nothing to
                        choose from; `Springfield · Illinois · United States`
                        is an answer. */}
                    <span class="settings__hint">
                      {city.region ? `${city.region} · ` : ""}
                      {city.country}
                    </span>
                  </button>
                </li>
              )}
            </For>
          </ul>
        </Show>
      </Show>
    </>
  );
}

/** The settings a picked city writes. Shared for the same reason the search is. */
export function placeFromCity(settings: Settings, city: City): Settings {
  return {
    ...settings,
    location: {
      ...settings.location,
      mode: "manual",
      place: {
        label: city.city,
        zone: city.zone,
        latitude: city.latitude,
        longitude: city.longitude,
        // A height, at last. The old table had no elevation column, so this was
        // always `null` - which was the honest answer then, because writing zero
        // would have claimed sea level for every city in the world. GeoNames
        // carries one from a digital elevation model, so a picked city now
        // brings its own. Still `null` where even that has none, which is not
        // zero. The user's own correction lives beside the place and continues
        // to apply, so it is not copied here.
        elevation: city.elevation,
      },
    },
  };
}

function Location(props: SectionProps): JSX.Element {
  const [locating, setLocating] = createSignal(false);

  // `undefined` until the field is touched, so the inputs show the location in
  // force rather than an empty box, and stop showing it the moment the user
  // starts typing their own.
  const [typedLatitude, setTypedLatitude] = createSignal<string | undefined>();
  const [typedLongitude, setTypedLongitude] = createSignal<
    string | undefined
  >();
  const [coordinateError, setCoordinateError] = createSignal<string>();

  // Forgotten whenever the location in force changes by any other route.
  //
  // They were kept, so typing a latitude and then searching and picking a city
  // left the two boxes showing the typed numbers - which were no longer the
  // place the app was using. The fields claimed to show the location in force
  // and did not.
  createEffect(
    on(
      () => [props.boot.location.latitude, props.boot.location.longitude],
      () => {
        setTypedLatitude(undefined);
        setTypedLongitude(undefined);
        setCoordinateError(undefined);
      },
      { defer: true },
    ),
  );

  async function applyCoordinates() {
    const latitude = Number(typedLatitude() ?? props.boot.location.latitude);
    const longitude = Number(typedLongitude() ?? props.boot.location.longitude);

    // Both, or neither. Rejecting rather than clamping: a latitude of 91 is a
    // typing mistake and not a request to stand at the pole, and silently
    // moving somebody 1° is worse than telling them.
    const valid =
      Number.isFinite(latitude) &&
      Number.isFinite(longitude) &&
      Math.abs(latitude) <= 90 &&
      Math.abs(longitude) <= 180;
    if (!valid) {
      setCoordinateError("Latitude runs −90 to 90 and longitude −180 to 180.");
      return;
    }
    setCoordinateError(undefined);

    // The nearest city names the point. Only names it: the zone in force is
    // kept, because no precision recovers a political boundary from a point
    // (D-031). `null` means the coordinates were not on the globe, which the
    // check above has already excluded, so the fallback is for safety only.
    const nearest = await ipc
      .nearestCity(latitude, longitude)
      .catch(() => null);

    props.apply({
      ...settings(),
      location: {
        ...settings().location,
        mode: "manual",
        place: {
          label: nearest?.city ?? props.boot.location.label,
          zone: props.boot.location.zone,
          latitude,
          longitude,
          // The coordinates are the user's; the height is not something they
          // implied by typing them. The nearest city's would be a guess about a
          // place they have just said they are not in.
          elevation: null,
        },
      },
    });
  }
  const [searching, setSearching] = createSignal(false);
  const settings = () => props.boot.settings;

  return (
    <div class="settings__section">
      <div class="settings__current">
        <span class="settings__current-place">{props.boot.location.label}</span>
        <span class="settings__hint">
          {coordinates(props.boot.location)} ·{" "}
          {/* Not `0 m` when nobody knew. Rise and set are still computed at sea
              level, which is the assumption to make with no height - but saying
              `0 m` reported that assumption as a measurement. The old city table
              had no elevation column at all, so it said it for every city in the
              world; GeoNames carries one, so most places now know (E5). */}
          {props.boot.location.elevation_known
            ? `${props.boot.location.elevation.toFixed(0)} m`
            : "height not set"}{" "}
          · {provenanceLabel(props.boot.location.provenance)}
        </span>
      </div>

      <CitySearch
        onPick={(city) => props.apply(placeFromCity(settings(), city))}
        onSearchingChange={setSearching}
      />

      {/* Folded away while a result list is showing, so six cities do not push
          the elevation field and the buttons off the pane. */}
      <Show when={!searching()}>
        <div class="settings__row">
          <span class="settings__label">Elevation</span>
          <input
            class="settings__number"
            type="number"
            step="10"
            placeholder="not set"
            // The correction the user typed, not the resolved elevation.
            // Showing the resolved figure put a number in a field the user had
            // left empty, so clearing it looked like it had not worked.
            value={settings().location.elevation ?? ""}
            onChange={(event) => {
              // The observer's own correction, not part of the place. Writing a
              // whole place around it made a timezone-derived location report
              // itself as having come from this Mac.
              const typed = event.currentTarget.value.trim();
              props.apply({
                ...settings(),
                location: {
                  ...settings().location,
                  // Empty means "I do not know", not zero. `Number("")` is 0,
                  // so clearing the field used to assert sea level - the same
                  // confusion `Option<f64>` exists to prevent, arriving from
                  // the keyboard.
                  elevation: typed === "" ? null : Number(typed),
                },
              });
            }}
          />
        </div>

        {/* Coordinates typed by hand: the last resort for somewhere no dataset
            has, and the only way to be exact. 34,129 cities is a great many more
            than 448, and it is still every place over 15,000 people - a village
            is not in it, and somebody living in one has no other way to say so.

            Applied only when both fields parse and both are on the globe. A
            half-typed latitude is not a location, and writing one as it is typed
            would move the observer to the equator between keystrokes.

            The zone is not touched. A zone boundary is political and is not
            recoverable from a point (D-031), so these coordinates keep whatever
            zone is already in force; only the name comes from the nearest city,
            which is a label and nothing more. */}
        <div class="settings__row">
          <span class="settings__label">Coordinates</span>
          <input
            class="settings__number"
            type="number"
            step="0.0001"
            placeholder="latitude"
            aria-label="Latitude, degrees north"
            value={typedLatitude() ?? props.boot.location.latitude.toFixed(4)}
            onInput={(event) => setTypedLatitude(event.currentTarget.value)}
            onChange={() => applyCoordinates()}
          />
          <input
            class="settings__number"
            type="number"
            step="0.0001"
            placeholder="longitude"
            aria-label="Longitude, degrees east"
            value={typedLongitude() ?? props.boot.location.longitude.toFixed(4)}
            onInput={(event) => setTypedLongitude(event.currentTarget.value)}
            onChange={() => applyCoordinates()}
          />
        </div>

        <Show when={coordinateError()}>
          {(problem) => <p class="settings__empty">{problem()}</p>}
        </Show>

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
                void ipc
                  .requestDeviceLocation()
                  .finally(() => setLocating(false));
              }}
            >
              {locating() ? "Asking macOS…" : "Use this Mac"}
            </button>
          </Show>
          {/* "Use my timezone" was here. It cleared the stored place, which
              since D-029 means having no location at all - so it dropped the
              user straight back behind the unskippable gate, with no way to
              undo it except to pick a city, which is what they had. A control
              whose only outcome is a state the app refuses to run in is not a
              choice. */}
        </div>
      </Show>
    </div>
  );
}

function provenanceLabel(
  provenance: Bootstrap["location"]["provenance"],
): string {
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
    <div class="settings__section">
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

/**
 * The optional limbs of the day view.
 *
 * Only the three that cost something are here. Dignity, drishti, planetary war
 * and the nakshatra lord are always on: they cost one positions call between
 * them, and a switch for a field that is free is a decision asked of the user
 * for nothing.
 *
 * A limb that is off is not computed rather than computed and hidden, so these
 * are not display preferences - they are what the day is asked for.
 */
function Panchanga(props: SectionProps): JSX.Element {
  const settings = () => props.boot.settings;

  function set(limb: keyof Settings["panchanga"], on: boolean) {
    props.apply({
      ...settings(),
      panchanga: { ...settings().panchanga, [limb]: on },
    });
  }

  return (
    <div class="settings__section">
      <Toggle
        label="Yogas"
        on={settings().panchanga.yogas}
        onToggle={() => set("yogas", !settings().panchanga.yogas)}
      />
      <Toggle
        label="Karanas"
        on={settings().panchanga.karanas}
        onToggle={() => set("karanas", !settings().panchanga.karanas)}
      />
      <Toggle
        label="Muhurtas"
        on={settings().panchanga.muhurtas}
        onToggle={() => set("muhurtas", !settings().panchanga.muhurtas)}
      />

      <p class="settings__hint settings__hint--foot">
        Shown in the day view, under Day. Rahu Kaal, Yamaganda, Gulika, Abhijit,
        Brahma Muhurta and Durmuhurtam are the muhurtas.
      </p>

      <p class="settings__hint settings__hint--foot">
        {/* So the absence of a switch for these does not read as an omission. */}
        Dignity, drishti, planetary war and the nakshatra lord are always shown.
      </p>
    </div>
  );
}

/**
 * The Lagna Kundali: which division it draws, and in which format.
 *
 * All three common formats, because they differ only in where a rashi is drawn.
 * North Indian is the default: it is the one most likely to be recognised, and
 * it is the only one that cannot be drawn without a lagna - which is why a
 * location is now required (D-029).
 */
function Chart(props: SectionProps): JSX.Element {
  const settings = () => props.boot.settings;

  return (
    <div class="settings__section">
      <p class="settings__hint settings__hint--foot">
        Always in the menu bar, at the right of the row. The chart it opens is
        where the nine grahas stand now, not a birth chart.
      </p>

      {/* Switches, not a choice. Several divisions can be in the menu bar at
          once, because a practitioner reads D1 and D9 together and flipping
          between them to compare is not reading them together. Each one on gets
          its own status item and its own panel, exactly as each graha does.

          D1 has no switch. It is the rashi chart, the one D-030 made permanent,
          and the chart the others are divisions *of*. */}
      <For each={props.boot.vargas}>
        {(choice) => {
          const permanent = choice.key === "d1";
          const on = () =>
            permanent ||
            settings().chart.vargas.includes(choice.key as VargaKey);
          return (
            <button
              class="settings__toggle settings__toggle--division"
              classList={{ "is-on": on() }}
              disabled={permanent}
              aria-pressed={on()}
              onClick={() => {
                const chosen = new Set(settings().chart.vargas);
                if (on()) chosen.delete(choice.key as VargaKey);
                else chosen.add(choice.key as VargaKey);
                chosen.add("d1");
                props.apply({
                  ...settings(),
                  chart: { ...settings().chart, vargas: [...chosen] },
                });
              }}
            >
              {/* The number, the name, the switch. What each division is read
                  for was here too and is gone: sixteen rows each carrying a
                  phrase is a paragraph to scan rather than a list, and the names
                  are what a reader picks by. */}
              <span class="settings__division">D{choice.division}</span>
              <span class="settings__toggle-name">{choice.name}</span>
              <span class="settings__switch" aria-hidden="true" />
            </button>
          );
        }}
      </For>

      <p class="settings__hint settings__hint--foot">
        Each division on gets its own menu bar item. D1 is always shown.
      </p>

      <ChoiceGroup label="Format">
        <For each={CHART_FORMATS}>
          {(choice) => (
            <Choice
              label={choice.label}
              selected={settings().chart.format === choice.key}
              onSelect={() =>
                props.apply({
                  ...settings(),
                  chart: { ...settings().chart, format: choice.key },
                })
              }
            />
          )}
        </For>
      </ChoiceGroup>

      <p class="settings__hint settings__hint--foot">
        North Indian fixes the houses and moves the signs through them, so the
        rising sign is always the top compartment. South and East Indian fix the
        signs and mark the rising one with a stroke.
      </p>
    </div>
  );
}

const CHART_FORMATS: { key: ChartFormat; label: string }[] = [
  { key: "north", label: "North Indian" },
  { key: "south", label: "South Indian" },
  { key: "east", label: "East Indian" },
];

/**
 * Everything that changes the model or the notation.
 *
 * The split is by who sets it, not by how obscure it is. General holds what a
 * normal user picks - where they are, which calendar, how big the panel. This
 * holds what changes *what is computed* (the ayanamsa, the node type, which
 * panchanga limbs) or *how it is written* (ingress labels, chart compartments).
 *
 * Ayanamsa and node type are the strongest case: they move every figure in the
 * app by up to a whole rashi, and they sat between "Menu bar" and "Size" as
 * though they were a preference about the interface.
 */
/**
 * Whether the chart moves.
 *
 * Its own pane rather than a row in the chart's, because it is about how the
 * chart behaves and not about what it draws - and because a reader who wants it
 * off wants to find it once and never again.
 */
function Motion(props: SectionProps): JSX.Element {
  const settings = () => props.boot.settings;

  return (
    <div class="settings__section">
      <Toggle
        label="Follow the lagna"
        on={settings().chart.animate}
        onToggle={() =>
          props.apply({
            ...settings(),
            chart: { ...settings().chart, animate: !settings().chart.animate },
          })
        }
      />

      <p class="settings__hint settings__hint--foot">
        The compartments' contents slide as the lagna crosses its sign, so a
        compartment against its far wall is one about to hand over. A system
        setting asking for reduced motion turns this off whatever is chosen here.
      </p>
    </div>
  );
}

/**
 * Whether the chart draws the lines it is laid out on.
 *
 * Its own pane and not a row in the chart's, for the same reason Motion has
 * one: it is about how the chart is drawn rather than about what it says, and
 * the explanation is longer than a row can carry.
 *
 * Off by default, unlike every other switch here that ships on. The grid is the
 * chart's own scaffolding; a reader who has not asked for it has asked for a
 * chart, and drawing construction lines over it unbidden would be answering a
 * question nobody put.
 */
function Grid(props: SectionProps): JSX.Element {
  const settings = () => props.boot.settings;

  return (
    <div class="settings__section">
      <Toggle
        label="Show the degree lines"
        on={settings().chart.grid}
        onToggle={() =>
          props.apply({
            ...settings(),
            chart: { ...settings().chart, grid: !settings().chart.grid },
          })
        }
      />

      <p class="settings__hint settings__hint--foot">
        Every North Indian chart is laid out this way whether the lines are
        drawn or not: each compartment runs from the edge a sign arrives through
        to the edge it leaves by, divided into the thirty degrees of that sign,
        with parallel lines either side for bodies that share a degree. This
        shows the scale the bodies are already standing on — why two grahas in
        one sign sit where they do, and how much room a crowded house has left.
        The other two formats have no route through a compartment, so there is
        nothing to draw.
      </p>
    </div>
  );
}

function Sky(props: SectionProps): JSX.Element {
  const settings = () => props.boot.settings;

  return (
    <div class="settings__section">
      <Toggle
        label="Show the stars"
        on={settings().chart.sky}
        onToggle={() =>
          props.apply({
            ...settings(),
            chart: { ...settings().chart, sky: !settings().chart.sky },
          })
        }
      />

      <p class="settings__hint settings__hint--foot">
        A field of stars behind the compartments, thinned toward the middle
        where the bodies stand. It is the only decoration in the chart
        and it carries no reading: no star marks anything, and turning it off
        changes nothing but the look. The field is the same every time — it is
        drawn from a fixed pattern, not scattered afresh. It drifts very slowly;
        a system asking for reduced motion stills it.
      </p>
    </div>
  );
}

function Advanced(props: {
  boot: Bootstrap;
  onOpen: (section: SettingsSection) => void;
}): JSX.Element {
  const settings = () => props.boot.settings;

  const ayanamsaLabel = () =>
    props.boot.ayanamsas.find(
      (choice) => choice.key === settings().sidereal.ayanamsa,
    )?.label ?? "";

  const rows: { id: SettingsSection; value: () => string }[] = [
    { id: "astrology", value: () => ayanamsaLabel().split(" ")[0] ?? "" },
    { id: "panchanga", value: () => limbCount(settings()) },
    {
      id: "ingress",
      value: () =>
        INGRESS_CHOICES.find((c) => c.key === settings().calendar.ingress)
          ?.short ?? "",
    },
    {
      id: "compartments",
      value: () => (settings().chart.numbered ? "Numbers" : "Names"),
    },
    {
      id: "motion",
      value: () => (settings().chart.animate ? "On" : "Off"),
    },
    {
      id: "grid",
      value: () => (settings().chart.grid ? "On" : "Off"),
    },
    {
      id: "sky",
      value: () => (settings().chart.sky ? "On" : "Off"),
    },
  ];

  return (
    <nav class="settings__list">
      {/* Said where the choices are, because that is what it is about: the
          settings on this screen are the defaults, not the reader's. The file
          is left untouched rather than reset, so this stays until they fix it
          or change something - which overwrites it deliberately. */}
      <Show when={props.boot.settings_error}>
        {(cause) => (
          <div class="error-block">
            <p class="error-block__headline">
              Your saved settings could not be read.
            </p>
            <p class="error-block__cause">{cause()}</p>
            <p class="error-block__cause">
              Chandra is running on defaults. The file has been left as it is;
              changing any setting here will overwrite it.
            </p>
          </div>
        )}
      </Show>
      <For each={rows}>
        {(row) => (
          <button class="settings__nav" onClick={() => props.onOpen(row.id)}>
            <span class="settings__nav-title">{SECTION_TITLES[row.id]}</span>
            <span class="settings__nav-value">{row.value()}</span>
            <Chevron />
          </button>
        )}
      </For>
    </nav>
  );
}

/** Which ingress the grid labels, if either. */
function Ingress(props: SectionProps): JSX.Element {
  const settings = () => props.boot.settings;

  return (
    <div class="settings__section">
      <ChoiceGroup label="Ingress labels">
        <For each={INGRESS_CHOICES}>
          {(choice) => (
            <Choice
              label={choice.label}
              selected={settings().calendar.ingress === choice.key}
              onSelect={() =>
                props.apply({
                  ...settings(),
                  calendar: { ...settings().calendar, ingress: choice.key },
                })
              }
            />
          )}
        </For>
      </ChoiceGroup>
      <p class="settings__hint settings__hint--foot">
        On the day a graha enters a sign or a nakshatra, its cell names what it
        entered instead of drawing the glyph. Not on the Moon's calendar: it
        enters a nakshatra every day, so every cell would be a label and none of
        them would be a phase.
      </p>
    </div>
  );
}

/** What a North Indian compartment is captioned with. */
function Compartments(props: SectionProps): JSX.Element {
  const settings = () => props.boot.settings;

  return (
    <div class="settings__section">
      <ChoiceGroup label="North Indian compartments">
        <Choice
          label="Names"
          selected={!settings().chart.numbered}
          onSelect={() =>
            props.apply({
              ...settings(),
              chart: { ...settings().chart, numbered: false },
            })
          }
        />
        <Choice
          label="Numbers"
          selected={settings().chart.numbered}
          onSelect={() =>
            props.apply({
              ...settings(),
              chart: { ...settings().chart, numbered: true },
            })
          }
        />
      </ChoiceGroup>
      <p class="settings__hint settings__hint--foot">
        Drik Panchang and Jagannatha Hora both write a number here. A number is
        a lookup, so this app writes the name and offers the number. The South
        and East Indian formats are unaffected: their compartments are the
        signs, and both references name them.
      </p>
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
    <div class="settings__section">
      <For each={props.boot.grahas}>
        {(graha) => {
          const on = () => settings().tray.subjects.includes(graha.key);
          return (
            <button
              class="settings__toggle"
              classList={{ "is-on": on() }}
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
        The chart is always shown; every calendar here can be switched off.
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
      {/* Served, not written here. A hardcoded version desynchronises at the
          next bump and nothing catches it, because nothing compares the two. */}
      <div class="settings__row">
        <span class="settings__label">Chandra</span>
        <span class="settings__value">{props.boot.app_version}</span>
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
      {/* Required, not courtesy. The city list is GeoNames under CC BY 4.0,
          whose one condition is that the credit appears where the work is used -
          so it is on a pane a reader can reach and not only in a file in the
          repository. */}
      <div class="settings__row">
        <span class="settings__label">Cities</span>
        <span class="settings__value">GeoNames, CC BY 4.0</span>
      </div>
      {/* The copyright notice, not just the licence name. Astrodienst's own
          conditions require that "the copyright notices and this notice be
          preserved on all copies" - a line saying AGPL-3.0 satisfies neither.
          The full text and the reproduced conditions ship beside the binary in
          the bundle's Resources; this is the part a reader can see.

          Their names appear here and nowhere promotional, which is the other
          half of the same clause: the trademark may be used to promote a
          product, the authors' and the copyright holder's names may not. */}
      <p class="settings__hint">
        Swiss Ephemeris © 1997–2021 Astrodienst AG. Chandra is not affiliated
        with Astrodienst.
      </p>
      <p class="settings__hint settings__hint--foot">
        Positions are computed on this Mac. Nothing is sent anywhere.
      </p>

      {/* Both open a browser rather than doing the thing here.
          Feedback in particular: an in-app form would need network code, and
          the line directly above would stop being true. The claim is worth more
          than the few people who will not follow a link - this app holds
          somebody's home coordinates, and "nothing is sent anywhere" is the
          whole of why that is safe to give it. */}
      <div class="settings__links">
        <button
          class="settings__link"
          onClick={() => void ipc.openLink("feedback")}
        >
          Send feedback
        </button>
        <button
          class="settings__link"
          onClick={() => void ipc.openLink("support")}
        >
          Support the work
        </button>
        <button
          class="settings__link"
          onClick={() => void ipc.openLink("source")}
        >
          Source code
        </button>
      </div>
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
function ChoiceGroup(props: {
  label: string;
  children: JSX.Element;
}): JSX.Element {
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
/**
 * The coordinates, printed to the precision the data actually has.
 *
 * `zone.tab` stores degrees and arcminutes - `+2232+08822` for Kolkata - so
 * every city and every timezone centroid in the app is known to about 1.9 km.
 * Three decimals of a degree is 110 m, so the last one was a unit conversion
 * rather than information, and it was claiming seventeen times the precision
 * the file holds. Two decimals is about 1.1 km, which is the honest rounding.
 *
 * CoreLocation is the exception: those are the device's own coordinates and are
 * as precise as they read, so they keep the third decimal.
 */
function coordinates(location: Resolved): string {
  const places = location.provenance === "core_location" ? 3 : 2;
  return `${location.latitude.toFixed(places)}, ${location.longitude.toFixed(places)}`;
}

/** `Off`, `Yogas`, `2 of 3`, `All`. */
function limbCount(settings: Settings): string {
  const limbs = settings.panchanga;
  const on = [
    limbs.yogas && "Yogas",
    limbs.karanas && "Karanas",
    limbs.muhurtas && "Muhurtas",
  ].filter((label): label is string => typeof label === "string");

  if (on.length === 0) return "Off";
  if (on.length === 1) return on[0]!;
  if (on.length === 3) return "All";
  return `${on.length} of 3`;
}

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
