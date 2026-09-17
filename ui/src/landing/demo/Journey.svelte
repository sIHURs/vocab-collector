<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import * as Dialog from '$lib/components/ui/dialog';
  import WindowsApp from '../../windows/WindowsApp.svelte';
  import WindowsFloatingCapture from '../../windows/WindowsFloatingCapture.svelte';
  import { LandingSession, createCaptureAdapter } from './session';
  import { channel, isThemeMessage, type DemoMessage } from '../protocol';
  export let session = new LandingSession();
  let captureOpen = false;
  let trigger: HTMLButtonElement | null = null;
  let captureBackend = createCaptureAdapter(session, closeCapture);
  function closeCapture() { captureOpen = false; }
  function openCapture() {
    captureBackend = createCaptureAdapter(session, closeCapture);
    captureOpen = true;
  }
  onMount(() => {
    const receive = (event: MessageEvent) => {
      if (event.origin !== location.origin || event.source !== window.parent || !isThemeMessage(event.data)) return;
      document.documentElement.dataset.appearance = event.data.theme;
    };
    window.addEventListener('message', receive);
    window.parent.postMessage({ channel, type: 'ready' } satisfies DemoMessage, location.origin);
    return () => window.removeEventListener('message', receive);
  });
</script>

<section class="reading" aria-label="Reading sample">
  <h2>A quiet garden</h2>
  <p>The <Button variant="link" bind:ref={trigger} onclick={openCapture}>ephemeral</Button> light made the garden feel new.</p>
  <p>Choose the highlighted word, then explicitly save it. Translations are prepared examples; no service is contacted.</p>
</section>
<WindowsApp api={session} embedded />
<Dialog.Root bind:open={captureOpen}>
  <Dialog.Content class="landing-capture-dialog" showCloseButton={false}
    onOpenAutoFocus={(event) => { event.preventDefault(); void tick().then(() => document.querySelector<HTMLButtonElement>('.landing-capture-dialog button')?.focus()); }}
    onCloseAutoFocus={(event) => { event.preventDefault(); trigger?.focus(); }}>
    <Dialog.Title class="sr-only">Capture a word</Dialog.Title>
    <Dialog.Description class="sr-only">Save the selected word and its reading context to this demo session.</Dialog.Description>
    <WindowsFloatingCapture {captureBackend} embedded />
  </Dialog.Content>
</Dialog.Root>

<style>
  .reading { padding: 20px; color: var(--foreground); background: var(--background); }
  h2 { font-size: 1.125rem; font-weight: 600; }
  p { margin-top: 8px; }
  p:last-child { font-size: .8rem; color: var(--muted-foreground); }
</style>
