<script lang="ts">
  const productName = "Vocab Collector";
</script>

<main>
  <h1>{productName}</h1>
  <p>Local-first application foundation</p>
</main>

<style>
  :global(body) {
    margin: 0;
    font-family: Inter, ui-sans-serif, system-ui, sans-serif;
    background: #f3f4f7;
    color: rgba(20, 22, 28, 0.87);
  }

  main {
    min-height: 100vh;
    display: grid;
    place-content: center;
    text-align: center;
  }

  h1 {
    margin: 0;
    font-size: 28px;
  }

  p {
    color: #7b808a;
  }
</style>

