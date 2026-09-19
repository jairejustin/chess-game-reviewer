<script lang="ts">
  import EngineSettings from '$lib/components/ui/EngineSettings.svelte';
</script>

<main class="flat-page">
  <header class="flat-header">
    <h1>General Settings</h1>
  </header>
  
  <section class="flat-content settings-grid">
    <div class="settings-widget engine-widget">
      <header class="widget-header">
        <h2>Engine Configuration</h2>
      </header>
      <div class="widget-body">
        <EngineSettings />
      </div>
    </div>
  </section>
</main>

<style>
  .flat-page {
    height: 100%;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    gap: 2rem;
    background: #0f0f11;
  }

  .flat-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    color: #ececec;
    border-bottom: 1px solid #2a2a2e;
    padding: 0.5rem 1rem 0.5rem;
  }

  .flat-header h1 {
    font-family: 'Bebas Neue', sans-serif;
    font-size: 1.8rem;
    letter-spacing: 1px;
    margin: 0;
    transform: translateY(2px);
  }

  .flat-content {
    padding: 0 1rem 1rem 1rem;
    flex: 1;
    color: #aaa;
    overflow-y: auto;
  }

  .settings-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 1.5rem;
    grid-auto-flow: dense; 
    align-items: start;
  }

  .settings-widget {
    background: #161618;
    border: 1px solid #2a2a2e;
    border-radius: 8px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    width: 100%;
  }

  .widget-header {
    background: #1c1c1f;
    border-bottom: 1px solid #2a2a2e;
    padding: 0.75rem 1.25rem;
  }

  .widget-header h2 {
    font-family: 'Bebas Neue', sans-serif;
    font-size: 1.2rem;
    letter-spacing: 1px;
    color: #ececec;
    margin: 0;
    font-weight: 400;
  }

  .widget-body {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .engine-widget {
    max-width: 450px;
    justify-self: start;
    width: 100%;
  }

  @media (min-width: 650px) {
    .engine-widget {
      grid-column: span 2;
    }
  }
</style>