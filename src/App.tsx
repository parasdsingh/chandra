/** Loads the panel. There is only one window. */

import type { JSX } from "solid-js";
import { createResource, onCleanup, onMount, Show } from "solid-js";
import { listen } from "@tauri-apps/api/event";

import * as ipc from "./ipc";
import type { GrahaKey } from "./ipc/types";
import { Panel } from "./components/Panel";

/**
 * The visual harness, reachable at `?preview` in a dev build only.
 *
 * `import.meta.env.DEV` is statically false in a production build, so the branch
 * and everything it imports are dropped by the bundler and cannot ship.
 */
function isPreview(): boolean {
  return import.meta.env.DEV && window.location.search.includes("preview");
}

export function App(): JSX.Element {
  if (isPreview()) return <DevPreview />;
  return <PanelWindow />;
}

function DevPreview(): JSX.Element {
  const [module] = createResource(async () => {
    await import("./dev/preview.css");
    return (await import("./dev/preview")).Preview;
  });
  return <Show when={module()}>{(Preview) => Preview()()}</Show>;
}

/**
 * Which subject this page was opened for.
 *
 * Read from the URL the backend navigated to as it opened the panel, so it is
 * fixed for the life of the page and needs nothing to propagate.
 */
function subjectFromUrl(): GrahaKey {
  const value = new URLSearchParams(window.location.search).get("subject");
  return (value as GrahaKey | null) ?? "chandra";
}

function PanelWindow(): JSX.Element {
  const subject = subjectFromUrl();
  const [boot, { refetch, mutate }] = createResource(ipc.bootstrap);

  onMount(() => {
    const reload = () => void refetch();

    // A location arriving from CoreLocation changes every rise and set time, so
    // the panel refreshes rather than showing stale figures.
    const locationListener = listen("chandra://location", reload);
    onCleanup(() => void locationListener.then((unlisten) => unlisten()));
  });

  return (
    <Show
      when={boot()}
      fallback={
        // A failed bootstrap must say so. Rendering an empty fallback gives a
        // panel that opens onto nothing, with no way to tell whether it is
        // still loading or permanently broken.
        <Show when={boot.error} fallback={<div class="panel-frame" />}>
          <div class="panel-frame">
            <div class="panel">
              <div class="region">
                <div class="error-block">
                  <p class="error-block__headline">Chandra could not start.</p>
                  <p class="error-block__cause">{describe(boot.error)}</p>
                </div>
              </div>
            </div>
          </div>
        </Show>
      }
    >
      <Panel
        boot={boot()!}
        subject={subject}
        onSettingsApplied={(next) => mutate(next)}
      />
    </Show>
  );
}

function describe(error: unknown): string {
  if (typeof error === "object" && error !== null && "message" in error) {
    return String((error as { message: unknown }).message);
  }
  return String(error);
}
