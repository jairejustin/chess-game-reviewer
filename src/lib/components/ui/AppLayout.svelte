<script lang="ts">
  import ActionStrip from './ActionStrip.svelte';
</script>

<main class="layout">
  <section class="layout__board">
    <slot name="board" />
  </section>

  <ActionStrip />

  <slot name="sidebar" />
</main>

<style>
  .layout {
    display: flex;
    height: 100vh;
    width: 100vw;
    max-width: 100%;
    margin: 0;
    padding: 1rem;
    gap: 0;
    box-sizing: border-box;
    overflow: hidden;
    align-items: flex-start;
  }

  .layout__board {
    flex: 1;
    display: flex;
    justify-content: center;
    align-items: center;
    height: 100%;
    min-height: 0;
    padding: 4rem 1rem 4rem 4rem;
    box-sizing: border-box;
  }
</style>
