/** Loads the panel. There is only one window. */

import type { JSX } from "solid-js";
import { createResource, onCleanup, onMount, Show } from "solid-js";
import { listen } from "@tauri-apps/api/event";

import * as ipc from "./ipc";
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

/** Release screenshots, at `?preview&shots`. Its own mode because the harness
 *  draws forty animated panels, which is more than a browser can be driven
 *  through for a picture - and a screenshot wants one panel, on a plain ground,
 *  at a size worth publishing. */
function isShots(): boolean {
  return isPreview() && window.location.search.includes("shots");
}

export function App(): JSX.Element {
  if (isShots()) return <DevShots />;
  if (isPreview()) return <DevPreview />;
  return <PanelWindow />;
}

function DevShots(): JSX.Element {
  const [module] = createResource(async () => {
    await import("./dev/preview.css");
    await import("./dev/shots.css");
    const { Shots } = await import("./dev/shots");
    const { previewBoot } = await import("./dev/preview");
    return () => <Shots boot={previewBoot} />;
  });
  return <Show when={module()}>{(view) => view()()}</Show>;
}

function DevPreview(): JSX.Element {
  const [module] = createResource(async () => {
    await import("./dev/preview.css");
    return (await import("./dev/preview")).Preview;
  });
  return <Show when={module()}>{(Preview) => Preview()()}</Show>;
}

function PanelWindow(): JSX.Element {
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
      <Panel boot={boot()!} onSettingsApplied={(next) => mutate(next)} />
    </Show>
  );
}

function describe(error: unknown): string {
  if (typeof error === "object" && error !== null && "message" in error) {
    return String((error as { message: unknown }).message);
  }
  return String(error);
}
