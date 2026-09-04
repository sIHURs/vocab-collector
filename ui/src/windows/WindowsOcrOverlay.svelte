<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";
  import type { RegionOcrStart } from "./captureBackend";

  let requestId = "";
  let start: { x: number; y: number } | null = null;
  let current: { x: number; y: number } | null = null;
  let selecting = false;

  $: rectangle = start && current ? {
    x: Math.min(start.x, current.x), y: Math.min(start.y, current.y),
    width: Math.abs(current.x - start.x), height: Math.abs(current.y - start.y),
  } : null;

  function begin(event: MouseEvent) {
    if (!requestId || event.button !== 0) return;
    start = { x: event.clientX, y: event.clientY };
    current = start;
    selecting = true;
  }

  function move(event: MouseEvent) {
    if (selecting) current = { x: event.clientX, y: event.clientY };
  }

  async function finish() {
    if (!selecting || !start || !current) return;
    selecting = false;
    const selected = {
      x: Math.min(start.x, current.x), y: Math.min(start.y, current.y),
      width: Math.abs(current.x - start.x), height: Math.abs(current.y - start.y),
    };
    if (selected.width < 4 || selected.height < 4) {
      start = null;
      current = null;
      return;
    }
    const window = getCurrentWindow();
    const [position, scale] = await Promise.all([window.outerPosition(), window.scaleFactor()]);
    try {
      await invoke("capture_ocr_region", { requestId, region: {
        x: position.x / scale + selected.x,
        y: position.y / scale + selected.y,
        width: selected.width,
        height: selected.height,
      }});
    } catch {
      await invoke("show_region_ocr_failure", { requestId });
    }
  }

  async function cancel() {
    if (requestId) await invoke("cancel_region_ocr_capture", { requestId });
  }

  onMount(() => {
    const ready = listen<RegionOcrStart>("region-ocr-start", ({ payload }) => {
      requestId = payload.requestId;
      start = null;
      current = null;
      selecting = false;
    });
    return () => { ready.then((unlisten) => unlisten()); };
  });
</script>

<svelte:window onkeydown={(event) => { if (event.key === "Escape") void cancel(); }} />
<button type="button" aria-label="Region OCR selection" onmousedown={begin} onmousemove={move} onmouseup={() => void finish()}>
  {#if !selecting}<div class="instruction" role="status"><strong>请框取一个词汇</strong><span>拖动鼠标框选 · Esc 取消</span></div>{/if}
  {#if rectangle}<div class="selection" style={`left:${rectangle.x}px;top:${rectangle.y}px;width:${rectangle.width}px;height:${rectangle.height}px`}></div>{/if}
</button>

<style>
  :global(html), :global(body), :global(#app) { width: 100%; height: 100%; margin: 0; overflow: hidden; background: transparent; }
  button { position: fixed; inset: 0; width: 100%; height: 100%; padding: 0; border: 0; cursor: crosshair; user-select: none; background: rgba(5, 7, 12, .42); }
  .instruction { position: fixed; top: 24px; left: 50%; display: grid; gap: 4px; padding: 12px 18px; border: 1px solid rgba(255,255,255,.25); border-radius: 8px; color: white; background: rgba(20,22,30,.92); font: 14px "Segoe UI", sans-serif; text-align: center; transform: translateX(-50%); pointer-events: none; }
  .instruction span { color: #c8cad4; font-size: 12px; }
  .selection { position: fixed; box-sizing: border-box; border: 2px solid #8793ff; background: rgba(255,255,255,.08); box-shadow: 0 0 0 9999px rgba(5,7,12,.28); pointer-events: none; }
  @media (forced-colors: active) { .selection { border-color: Highlight; } .instruction { border-color: CanvasText; color: CanvasText; background: Canvas; } }
</style>
