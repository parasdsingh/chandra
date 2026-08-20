/** Chooses which window this document is, and loads it. */

import type { JSX } from "solid-js";
import { createResource, createSignal, onCleanup, onMount, Show } from "solid-js";
import { listen } from "@tauri-apps/api/event";

import * as ipc from "./ipc";
import type { GrahaKey } from "./ipc/types";
import { Panel } from "./components/Panel";
import { SettingsWindow, useSettingsChrome } from "./components/Settings";

function isSettingsWindow(): boolean {
  return new URLSearchParams(window.location.search).get("window") === "settings";
}

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
  if (isPreview()) {
    return <DevPreview />;
  }
  return (
    <Show when={!isSettingsWindow()} fallback={<Settings />}>
      <PanelWindow />
    </Show>
  );
}

function DevPreview(): JSX.Element {
  const [module] = createResource(async () => {
    await import("./dev/preview.css");
    return (await import("./dev/preview")).Preview;
  });
  return <Show when={module()}>{(Preview) => Preview()()}</Show>;
}

function Settings(): JSX.Element {
  useSettingsChrome();
  return <SettingsWindow />;
}

function PanelWindow(): JSX.Element {
  const [subject, setSubject] = createSignal<GrahaKey>("chandra");
  const [boot, { refetch }] = createResource(ipc.bootstrap);

  onMount(() => {
    // The tray tells the panel which subject it was opened for; the window is
    // created once and reused for all of them.
    const subjectListener = listen<GrahaKey>("chandra://subject", (event) => {
      setSubject(event.payload);
    });
    // A location arriving from CoreLocation changes every rise and set time, so
    // the panel reloads rather than showing stale figures.
    const locationListener = listen("chandra://location", () => {
      void refetch();
    });

    onCleanup(() => {
      void subjectListener.then((unlisten) => unlisten());
      void locationListener.then((unlisten) => unlisten());
    });
  });

  return (
    <Show
      when={boot()}
      fallback={
        // A failed bootstrap must say so. Rendering the empty fallback instead
        // gives a panel that opens onto nothing, with no way to tell whether it
        // is still loading or permanently broken.
        <Show when={boot.error} fallback={<div class="panel-frame" />}>
          <div class="panel-frame">
            <div class="panel">
              <div class="detail">
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
      {(loaded) => <Panel boot={loaded()} subject={subject()} />}
    </Show>
  );
}

function describe(error: unknown): string {
  if (typeof error === "object" && error !== null && "message" in error) {
    return String((error as { message: unknown }).message);
  }
  return String(error);
}
