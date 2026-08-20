import { render } from "solid-js/web";

import { App } from "./App";
import "./styles/tokens.css";
import "./styles/base.css";
import "./styles/panel.css";
import "./styles/settings.css";

const root = document.getElementById("root");
if (!root) {
  throw new Error("index.html must contain #root");
}

render(() => <App />, root);
