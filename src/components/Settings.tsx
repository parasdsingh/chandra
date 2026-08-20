/**
 * The settings window.
 *
 * Five sections, matching ARCHITECTURE section 7. Every change is applied
 * immediately - there is no Save button and no confirmation, because every
 * setting here is reversible and instantly visible.
 */

import type { JSX } from "solid-js";
import {
  createEffect,
  createResource,
  createSignal,
  For,
  onMount,
  Show,
} from "solid-js";

import * as ipc from "../ipc";
import type { Bootstrap, City, GrahaKey, Settings as Model } from "../ipc/types";
import { GrahaGlyph } from "./GrahaGlyph";

type Tab = "general" | "location" | "astrology" | "grahas" | "about";

const TABS: { id: Tab; label: string }[] = [
  { id: "general", label: "General" },
  { id: "location", label: "Location" },
  { id: "astrology", label: "Astrology" },
  { id: "grahas", label: "Grahas" },
  { id: "about", label: "About" },
];

/** Fetches and persists; rendering is [`SettingsView`]. */
export function SettingsWindow(): JSX.Element {
  const [boot, { mutate }] = createResource(ipc.bootstrap);
  const [saving, setSaving] = createSignal(false);
  const [failure, setFailure] = createSignal<string>();

  async function apply(next: Model) {
    setSaving(true);
    setFailure(undefined);
    try {
      mutate(await ipc.updateSettings(next));
    } catch (thrown) {
      setFailure(
        typeof thrown === "object" && thrown && "message" in thrown
          ? String((thrown as { message: string }).message)
          : String(thrown),
      );
    } finally {
      setSaving(false);
    }
  }

  return (
    <SettingsView
      boot={boot()}
      apply={(next) => void apply(next)}
      busy={saving()}
      failure={failure()}
    />
  );
}

/**
 * Presentation only.
 *
 * Split from the fetching so the visual harness can render it against real
 * data: a settings pane that can only be seen by launching the app is a pane
 * whose layout never gets checked.
 */
export function SettingsView(props: {
  boot: Bootstrap | undefined;
  apply: (next: Model) => void;
  busy: boolean;
  failure: string | undefined;
  initialTab?: Tab;
}): JSX.Element {
  const [tab, setTab] = createSignal<Tab>(props.initialTab ?? "general");

  return (
    <div class="settings">
      <nav class="settings__tabs" role="tablist">
        <For each={TABS}>
          {(entry) => (
            <button
              class="settings__tab"
              classList={{ "is-active": tab() === entry.id }}
              role="tab"
              aria-selected={tab() === entry.id}
              onClick={() => setTab(entry.id)}
            >
              {entry.label}
            </button>
          )}
        </For>
      </nav>

      <Show when={props.boot} fallback={<div class="settings__body" />}>
        {(loaded) => (
          <div class="settings__body" role="tabpanel">
            <Show when={props.failure}>
              {(message) => <p class="settings__failure">{message()}</p>}
            </Show>

            <Show when={tab() === "general"}>
              <General boot={loaded()} apply={props.apply} busy={props.busy} />
            </Show>
            <Show when={tab() === "location"}>
              <Location boot={loaded()} apply={props.apply} />
            </Show>
            <Show when={tab() === "astrology"}>
              <Astrology boot={loaded()} apply={props.apply} />
            </Show>
            <Show when={tab() === "grahas"}>
              <Grahas boot={loaded()} apply={props.apply} />
            </Show>
            <Show when={tab() === "about"}>
              <About boot={loaded()} />
            </Show>
          </div>
        )}
      </Show>
    </div>
  );
}

interface SectionProps {
  boot: Bootstrap;
  apply: (next: Model) => void;
}

function Row(props: { label: string; children: JSX.Element }): JSX.Element {
  return (
    <div class="settings__row">
      <span class="settings__label">{props.label}</span>
      <span class="settings__control">{props.children}</span>
    </div>
  );
}

function General(props: SectionProps & { busy: boolean }): JSX.Element {
  const settings = () => props.boot.settings;
  return (
    <>
      <h2 class="settings__header">General</h2>
      <Row label="Launch at login">
        <input
          type="checkbox"
          checked={settings().launch_at_login}
          disabled={props.busy}
          onChange={(event) =>
            props.apply({
              ...settings(),
              launch_at_login: event.currentTarget.checked,
            })
          }
        />
      </Row>
      <Row label="Time format">
        <select
          value={settings().time_format}
          onChange={(event) =>
            props.apply({
              ...settings(),
              time_format: event.currentTarget.value as Model["time_format"],
            })
          }
        >
          <option value="system">Follow system</option>
          <option value="hour24">24 hour</option>
          <option value="hour12">12 hour</option>
        </select>
      </Row>
    </>
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
    void ipc.searchCities(text, 8).then(setResults);
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

  async function useDeviceLocation() {
    setLocating(true);
    try {
      await ipc.requestDeviceLocation();
    } finally {
      setLocating(false);
    }
  }

  return (
    <>
      <h2 class="settings__header">Location</h2>

      <Row label="In use">
        <span>
          {props.boot.location.label} · {props.boot.location.zone}{" "}
          <span class="settings__hint">
            ({provenanceLabel(props.boot.location.provenance)})
          </span>
        </span>
      </Row>

      <Row label="Coordinates">
        <span>
          {props.boot.location.latitude.toFixed(4)},{" "}
          {props.boot.location.longitude.toFixed(4)}
        </span>
      </Row>

      <Row label="Elevation">
        <input
          type="number"
          step="10"
          value={props.boot.location.elevation}
          onChange={(event) => {
            const place = settings().location.place;
            props.apply({
              ...settings(),
              location: {
                mode: settings().location.mode,
                place: place
                  ? { ...place, elevation: Number(event.currentTarget.value) }
                  : {
                      label: props.boot.location.label,
                      zone: props.boot.location.zone,
                      latitude: props.boot.location.latitude,
                      longitude: props.boot.location.longitude,
                      elevation: Number(event.currentTarget.value),
                    },
              },
            });
          }}
        />
        <span class="settings__hint">metres — shifts rise and set</span>
      </Row>

      <Row label="Use this Mac">
        <button
          class="settings__button"
          disabled={locating()}
          onClick={() => void useDeviceLocation()}
        >
          {locating() ? "Asking macOS…" : "Detect location"}
        </button>
      </Row>

      <Row label="Choose a city">
        <input
          type="search"
          placeholder="Search"
          value={query()}
          onInput={(event) => setQuery(event.currentTarget.value)}
        />
      </Row>

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

      <Show when={settings().location.mode === "manual"}>
        <Row label="">
          <button
            class="settings__button"
            onClick={() =>
              props.apply({
                ...settings(),
                location: { mode: "automatic", place: null },
              })
            }
          >
            Use my timezone instead
          </button>
        </Row>
      </Show>
    </>
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
  const [ayanamsa] = createResource(() => ipc.ayanamsaDegrees(Date.now()));

  return (
    <>
      <h2 class="settings__header">Astrology</h2>

      <Row label="Ayanamsa">
        <select
          value={settings().sidereal.ayanamsa}
          onChange={(event) =>
            props.apply({
              ...settings(),
              sidereal: {
                ...settings().sidereal,
                ayanamsa: event.currentTarget.value,
              },
            })
          }
        >
          <For each={props.boot.ayanamsas}>
            {(choice) => <option value={choice.key}>{choice.label}</option>}
          </For>
        </select>
      </Row>

      <Row label="Currently">
        <Show when={ayanamsa()} fallback={<span />}>
          {(degrees) => <span>{formatArc(degrees())}</span>}
        </Show>
      </Row>

      <Row label="Rahu and Ketu">
        <select
          value={settings().sidereal.node_type}
          onChange={(event) =>
            props.apply({
              ...settings(),
              sidereal: {
                ...settings().sidereal,
                node_type: event.currentTarget.value,
              },
            })
          }
        >
          <For each={props.boot.node_types}>
            {(choice) => <option value={choice.key}>{choice.label}</option>}
          </For>
        </select>
      </Row>
    </>
  );
}

function formatArc(degrees: number): string {
  const whole = Math.floor(degrees);
  const minutesTotal = (degrees - whole) * 60;
  const minutes = Math.floor(minutesTotal);
  const seconds = Math.round((minutesTotal - minutes) * 60);
  return `${whole}° ${minutes.toString().padStart(2, "0")}′ ${seconds
    .toString()
    .padStart(2, "0")}″`;
}

function Grahas(props: SectionProps): JSX.Element {
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
    <>
      <h2 class="settings__header">Menu bar</h2>

      <For each={props.boot.grahas}>
        {(graha) => (
          <div class="settings__graha">
            <GrahaGlyph info={graha} size={20} />
            <span class="settings__graha-name">{graha.name}</span>
            <span class="settings__hint">{graha.english}</span>
            <Show
              when={graha.key !== "chandra"}
              fallback={
                // The permanent moon item is Chandra's item; a second one would
                // put two moons in the menu bar (D-019).
                <span class="settings__hint">always shown</span>
              }
            >
              <input
                type="checkbox"
                checked={settings().tray.subjects.includes(graha.key)}
                onChange={(event) => toggle(graha.key, event.currentTarget.checked)}
              />
            </Show>
          </div>
        )}
      </For>

      <Row label="Coloured icons">
        <input
          type="checkbox"
          checked={settings().tray.colour_mode}
          onChange={(event) =>
            props.apply({
              ...settings(),
              tray: {
                ...settings().tray,
                colour_mode: event.currentTarget.checked,
              },
            })
          }
        />
        <span class="settings__hint">
          template icons follow the menu bar
        </span>
      </Row>
    </>
  );
}

function About(props: { boot: Bootstrap }): JSX.Element {
  return (
    <>
      <h2 class="settings__header">About</h2>
      <Row label="Chandra">
        <span>Version 0.1.0</span>
      </Row>
      <Row label="Ephemeris">
        <span>Swiss Ephemeris {props.boot.library_version}</span>
      </Row>
      <Row label="Data range">
        <span>1800–2399 at full precision</span>
      </Row>
      <Row label="Licence">
        <span>AGPL-3.0</span>
      </Row>
    </>
  );
}

/** Applies the window's own background, which the panel window leaves transparent. */
export function useSettingsChrome() {
  onMount(() => {
    document.body.classList.add("is-settings");
  });
}
