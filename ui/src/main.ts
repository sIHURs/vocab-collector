import { mount } from "svelte";
import App from "./App.svelte";
import FloatingCapture from "./FloatingCapture.svelte";
import "./styles.css";

const isCaptureWindow = new URLSearchParams(location.search).get("window") === "capture";
mount(isCaptureWindow ? FloatingCapture : App, { target: document.getElementById("app")! });
