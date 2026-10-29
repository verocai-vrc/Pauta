import { mount } from "svelte";
import "./app.css";
import App from "./App.svelte";
import Capture from "./components/Capture.svelte";

const isCapture = new URLSearchParams(location.search).has("capture");
mount(isCapture ? Capture : App, { target: document.getElementById("app")! });
