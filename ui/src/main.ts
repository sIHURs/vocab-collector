import { mount } from "svelte";
import { invoke } from "@tauri-apps/api/core";
import App from "./App.svelte";
import FloatingCapture from "./FloatingCapture.svelte";
import WindowsApp from "./windows/WindowsApp.svelte";
import WindowsFloatingCapture from "./windows/WindowsFloatingCapture.svelte";
import {
  selectPresentation,
  type DesktopWindow,
  type PresentationFamily,
} from "./lib/presentation";
import "./styles.css";

function desktopWindow(): DesktopWindow {
  return new URLSearchParams(location.search).get("window") === "capture" ? "capture" : "main";
}

async function presentationFamily(): Promise<PresentationFamily> {
  if (!("__TAURI_INTERNALS__" in globalThis)) return "shared";
  try {
    return await invoke<PresentationFamily>("get_presentation_family");
  } catch {
    return "shared";
  }
}

async function start() {
  const presentation = selectPresentation(desktopWindow(), await presentationFamily());
  const target = document.getElementById("app")!;

  switch (presentation) {
    case "windows-main": mount(WindowsApp, { target }); break;
    case "windows-capture": mount(WindowsFloatingCapture, { target }); break;
    case "shared-capture": mount(FloatingCapture, { target }); break;
    default: mount(App, { target });
  }
}

void start();
