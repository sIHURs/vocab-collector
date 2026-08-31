<script lang="ts">
  type Route = "Today" | "Vocabulary" | "Review" | "Settings";

  const navigation: Route[] = ["Today", "Vocabulary", "Review", "Settings"];
  let route: Route = "Today";
</script>

<div class="windows-shell" data-presentation="windows-main">
  <aside>
    <div class="brand"><span aria-hidden="true">V</span><strong>Vocab Collector</strong></div>
    <nav aria-label="Main navigation">
      {#each navigation as item}
        <button
          class:active={route === item}
          aria-current={route === item ? "page" : undefined}
          onclick={() => (route = item)}
        >{item}</button>
      {/each}
    </nav>
    <div class="local-status"><i aria-hidden="true"></i><span>Local mode</span></div>
  </aside>

  <main>
    <header><h1>{route}</h1></header>
    <section aria-label={`${route} content`}></section>
  </main>
</div>

<style>
  :global(html), :global(body), :global(#app) { min-width: 100%; min-height: 100%; margin: 0; }
  :global(body) { background: #15161c; }
  .windows-shell { min-height: 100vh; display: grid; grid-template-columns: 220px minmax(0, 1fr); color: #eeeef3; background: #15161c; font: 14px "Segoe UI Variable", "Segoe UI", sans-serif; }
  aside { display: flex; flex-direction: column; min-width: 0; padding: 18px 12px 14px; border-right: 1px solid #2c2e38; background: #191a21; }
  .brand { display: flex; align-items: center; gap: 10px; min-height: 36px; padding: 0 8px 18px; }
  .brand > span { display: grid; place-items: center; width: 28px; height: 28px; border-radius: 6px; background: #7584ef; color: #fff; font-weight: 700; }
  .brand strong { font-size: 13px; font-weight: 650; }
  nav { display: grid; gap: 3px; }
  nav button { min-height: 36px; padding: 0 10px; border: 0; border-radius: 6px; color: #a9acb8; background: transparent; text-align: left; cursor: pointer; }
  nav button:hover { color: #eeeef3; background: #232530; }
  nav button.active { color: #fff; background: #2a2d3b; }
  nav button:focus-visible { outline: 2px solid #8d9aff; outline-offset: 1px; }
  .local-status { display: flex; align-items: center; gap: 8px; margin-top: auto; padding: 10px 8px; color: #8f93a1; font-size: 12px; }
  .local-status i { width: 7px; height: 7px; border-radius: 50%; background: #78c8a7; }
  main { min-width: 0; }
  header { display: flex; align-items: center; height: 72px; padding: 0 28px; border-bottom: 1px solid #2c2e38; }
  h1 { margin: 0; font-size: 22px; font-weight: 650; letter-spacing: 0; }
  section { min-height: calc(100vh - 73px); }
</style>
