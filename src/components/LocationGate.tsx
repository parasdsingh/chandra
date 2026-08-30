/**
 * The first thing the app asks for, and the only thing it insists on.
 *
 * Chandra used to work from launch with no location, falling through D-007's
 * chain to the representative city of the system timezone. That was defensible
 * while the app showed phases: a phase is the same from anywhere.
 *
 * It is not defensible for what the app shows now. Sunrise, moonrise, the tithi
 * a day is named after and every muhurta are computed from where the observer
 * stands, and a centroid a thousand kilometres away gives a sunrise about forty
 * minutes wrong - enough to move the tithi at sunrise across a boundary. The
 * lagna is worse: one degree of longitude is four minutes is about one degree of
 * ascendant, so the same centroid names the wrong rashi roughly a quarter of the
 * time, and a Lagna Kundali is built around that one figure.
 *
 * So this blocks, where D-007 said nothing would. What it does not do is guess:
 * there is no "skip", because a skipped location is the guess this exists to
 * stop, and no default is offered for the same reason.
 */

import type { JSX } from "solid-js";
import { createSignal, Show } from "solid-js";

import * as ipc from "../ipc";
import type { Bootstrap, Settings } from "../ipc/types";
import { CitySearch, placeFromCity } from "./SettingsView";

interface Props {
  boot: Bootstrap;
  apply: (next: Settings) => void;
}

export function LocationGate(props: Props): JSX.Element {
  const [locating, setLocating] = createSignal(false);
  const [refused, setRefused] = createSignal(false);
  const [searching, setSearching] = createSignal(false);

  return (
    <div class="gate" role="region" aria-label="Set your location">
      <Show when={!searching()}>
        <p class="gate__lead">Chandra needs to know where you are.</p>
        <p class="gate__why">
          Sunrise, the tithi a day is named after, and the lagna are all
          different in different places. Without a location they would be a
          guess, and nothing on screen would say so.
        </p>
      </Show>

      <CitySearch
        onPick={(city) => props.apply(placeFromCity(props.boot.settings, city))}
        onSearchingChange={setSearching}
      />

      <Show when={!searching()}>
        <button
          class="settings__button gate__button"
          disabled={locating()}
          onClick={() => {
            setLocating(true);
            setRefused(false);
            void ipc
              .requestDeviceLocation()
              .then((resolved) => {
                // A refusal is a normal answer, not an error - macOS simply
                // does not say which it was. If the provenance did not change,
                // the ask did not land, and the city search below is the way
                // through rather than a dead end.
                if (resolved.provenance !== "core_location") setRefused(true);
              })
              .catch(() => setRefused(true))
              .finally(() => setLocating(false));
          }}
        >
          {locating() ? "Asking macOS…" : "Use this Mac"}
        </button>

        <Show when={refused()}>
          <p class="gate__refused">
            macOS did not give a location. Search for your city instead.
          </p>
        </Show>
      </Show>
    </div>
  );
}

/**
 * Whether a location has actually been set, as against fallen back to.
 *
 * `time_zone` is the last step of D-007's chain: the representative city of the
 * system timezone, which nobody chose and which can be a thousand kilometres
 * out. Every other provenance is an answer - `manual` because the user picked
 * it, `core_location` because the device supplied it.
 */
export function locationIsSet(boot: Bootstrap): boolean {
  return boot.location.provenance !== "time_zone";
}
