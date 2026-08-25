<script lang="ts">
  type Route = "Today" | "Vocabulary" | "Progress" | "Settings";
  let route: Route = "Today";

  const navigation: Array<{ label: Route; icon: string }> = [
    { label: "Today", icon: "◫" },
    { label: "Vocabulary", icon: "Aa" },
    { label: "Progress", icon: "↗" },
    { label: "Settings", icon: "⚙" },
  ];
</script>

<div class="window-shell">
  <aside class="sidebar">
    <div class="traffic-spacer" aria-hidden="true"></div>
    <div class="brand">
      <span class="brand-mark">V</span>
      <span>Vocab Collector</span>
    </div>

    <nav aria-label="Main navigation">
      {#each navigation as item}
        <button
          class:active={route === item.label}
          aria-current={route === item.label ? "page" : undefined}
          onclick={() => (route = item.label)}
        >
          <span class="nav-icon" aria-hidden="true">{item.icon}</span>
          {item.label}
        </button>
      {/each}
    </nav>

    <div class="account-card">
      <span class="status-dot"></span>
      <div><strong>Local mode</strong><small>Everything is saved</small></div>
    </div>
  </aside>

  <main>
    <header class="toolbar">
      <div>
        <h1>{route}</h1>
        <p>{route === "Today" ? "Your quiet learning rhythm" : `Manage your ${route.toLowerCase()}`}</p>
      </div>
      <div class="toolbar-actions">
        <button class="icon-button" aria-label="Search">⌕</button>
        <button class="avatar" aria-label="Account">YW</button>
      </div>
    </header>

    <section class="content" aria-label={`${route} content`}>
      <div class="placeholder-panel">
        <span class="eyebrow">{route}</span>
        <h2>{route === "Today" ? "Ready when you are." : `${route} at a glance`}</h2>
        <p>The product workflow will appear here in the next implementation milestone.</p>
      </div>
    </section>
  </main>
</div>
