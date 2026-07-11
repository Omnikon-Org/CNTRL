/* @refresh reload */
import { render } from "solid-js/web";
import App from "./App";
import { applyTheme, getStoredTheme } from "./utils/theme";

applyTheme(getStoredTheme());

render(() => <App />, document.getElementById("root") as HTMLElement);
