import { mount } from "svelte";
import "@fontsource-variable/fraunces/opsz.css";
import "@fontsource-variable/fraunces/opsz-italic.css";
import "@fontsource-variable/source-sans-3/wght.css";
import "@fontsource-variable/source-sans-3/wght-italic.css";
import "./app.css";
import App from "./App.svelte";
import Capture from "./components/Capture.svelte";

const isCapture = new URLSearchParams(location.search).has("capture");
mount(isCapture ? Capture : App, { target: document.getElementById("app")! });
