<script lang="ts">
  export let title: string;
  export let tabs: { id: string; label: string }[];
  export let activeTab: string;

  function setTab(id: string) {
    activeTab = id;
  }
</script>

<aside class="sidebar">
  <div class="sidebar__header">
    <h2 class="sidebar__title">{title}</h2>
  </div>

  {#if tabs.length > 0}
    <div class="sidebar__nav">
      {#each tabs as tab}
        <button
          class="sidebar__nav-btn"
          class:sidebar__nav-btn--active={activeTab === tab.id}
          on:click={() => setTab(tab.id)}
        >
          {tab.label}
        </button>
      {/each}
    </div>
  {/if}

  <div class="sidebar__content">
    <slot />
  </div>
</aside>

<style>
  .sidebar {
    width: 400px;
    height: 100%;
    max-height: 100%;
    flex-shrink: 0;
    background: var(--bg-panel, #161618);
    border: 1px solid var(--border-subtle, #2a2a2e);
    border-radius: 12px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
  }
  .sidebar__header {
    padding: 1.1rem 1.25rem 1rem;
    background: var(--bg-surface, #1c1c1f);
    border-bottom: 1px solid var(--border-subtle, #2a2a2e);
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    flex-shrink: 0;
  }
  .sidebar__title {
    font-family: 'Bebas Neue', sans-serif;
    font-size: 1.8rem;
    font-weight: 400;
    margin: 0;
    letter-spacing: 1px;
    color: #fff;
  }
  .sidebar__nav {
    display: flex;
    flex-shrink: 0;
    border-bottom: 1px solid var(--border-subtle, #2a2a2e);
    background: var(--bg-surface, #1c1c1f);
  }
  .sidebar__nav-btn {
    flex: 1;
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    color: #555;
    font-family: 'Bebas Neue', sans-serif;
    font-size: 1rem;
    font-weight: 400;
    letter-spacing: 2px;
    text-transform: uppercase;
    padding: 0.4rem 1rem;
    cursor: pointer;
    transition:
      color 0.15s ease,
      border-color 0.15s ease;
    margin-bottom: -1px;
  }
  .sidebar__nav-btn:hover:not(.sidebar__nav-btn--active) {
    color: #888;
  }
  .sidebar__nav-btn--active {
    color: #ececec;
    border-bottom-color: #ececec;
  }
  .sidebar__content {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow: hidden;
  }
</style>
