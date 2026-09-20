<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import Download from 'lucide-svelte/icons/download';
  import Settings from 'lucide-svelte/icons/settings';
</script>

<nav class="global-nav">
  <div class="nav-group">
    <button
      class="nav-btn"
      class:nav-btn--active={page.url.pathname.startsWith('/review') ||
        page.url.pathname.startsWith('/explorer')}
      on:click={() => goto('/review')}
      title="Analysis Workspace"
    >
      <!-- Custom Checkerboard SVG -->
      <svg
        width="26"
        height="26"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <rect x="3" y="3" width="18" height="18" rx="2.5" />
        <rect
          x="3"
          y="3"
          width="9"
          height="9"
          fill="currentColor"
          stroke="none"
          rx="1"
        />
        <rect
          x="12"
          y="12"
          width="9"
          height="9"
          fill="currentColor"
          stroke="none"
          rx="1"
        />
      </svg>
    </button>
    <button
      class="nav-btn"
      class:nav-btn--active={page.url.pathname.startsWith('/resources')}
      on:click={() => goto('/resources')}
      title="Resource Manager"
    >
      <Download size={26} strokeWidth={2} />
    </button>
  </div>

  <div class="nav-group">
    <button
      class="nav-btn"
      class:nav-btn--active={page.url.pathname.startsWith('/settings')}
      on:click={() => goto('/settings')}
      title="General Settings"
    >
      <Settings size={26} strokeWidth={2} />
    </button>
  </div>
</nav>

<style>
  .global-nav {
    width: 55px;
    height: 100vh;
    background: hsl(240, 3%, 7%);
    border-right: 1px solid #2a2a2e;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    box-sizing: border-box;
    flex-shrink: 0;
    z-index: 100;
  }

  .nav-group {
    display: flex;
    flex-direction: column;
    width: 100%;
  }

  .nav-btn {
    width: 100%;
    height: 52px;
    background: transparent;
    border: none;
    border: 1px solid transparent;
    color: #666;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition:
      color 0.15s ease,
      background 0.15s ease,
      border-color 0.15s ease;
    border-radius: 0;
  }

  .nav-btn:hover {
    color: #ececec;
    background: #161618;
  }

  .nav-btn--active {
    color: #8be1b4;
    background: rgba(139, 225, 180, 0.05);
    border-color: #8be1b4;
  }
</style>
