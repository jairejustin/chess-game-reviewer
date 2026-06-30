<script lang="ts">
  import {
    engineOn,
    engineStatus,
    livePVLines,
    toggleEngine,
    currentDepth,
    explorerIndex,
    currentLine,
    setExplorerIndex
  } from '$lib/stores/explorerStore';
  import MultiPVStack from './MultiPVStack.svelte';

  import MoveList from '../ui/MoveList.svelte';
  import NavigationControls from '../ui/NavigationControls.svelte';
  import Cpu from 'lucide-svelte/icons/cpu';
</script>

<aside class="sidebar">
  <div class="sidebar__header">
    <h2 class="sidebar__title">Live Explorer</h2>
  </div>

  <div class="engine-controls">
    <div class="engine-controls__left">
      <Cpu size={16} strokeWidth={2} />
      <span class="engine-controls__label">Stockfish</span>
    </div>

    <div class="engine-controls__right">
      <span
        class="engine-status"
        class:engine-status--running={$engineStatus === 'thinking'}
        class:engine-status--paused={$engineStatus === 'paused'}
        class:engine-status--starting={$engineStatus === 'starting'}
      >
        {#if $engineStatus === 'thinking'}
          Running
        {:else if $engineStatus === 'starting'}
          Starting…
        {:else}
          Paused
        {/if}
      </span>

      <button
        class="engine-toggle"
        class:engine-toggle--on={$engineOn}
        on:click={() => toggleEngine(!$engineOn)}
        title={$engineOn ? 'Pause engine' : 'Resume engine'}
        aria-label={$engineOn ? 'Pause engine' : 'Resume engine'}
      >
        <span class="engine-toggle__knob"></span>
      </button>
    </div>
  </div>

  <MultiPVStack
    lines={$livePVLines}
    active={$engineStatus === 'thinking'}
    status={$engineStatus}
    depth={$currentDepth}
  />

  <MoveList
    moves={$currentLine}
    activeIndex={$explorerIndex}
    onSelect={(i) => setExplorerIndex(i)}
    showBadges={false}
    emptyMessage="Make a move on the board."
  />

  <div class="sidebar__controls">
    <NavigationControls
      canGoBack={$explorerIndex > 0}
      canGoForward={$explorerIndex < $currentLine.length - 1}
      onBack={() => setExplorerIndex($explorerIndex - 1)}
      onForward={() => setExplorerIndex($explorerIndex + 1)}
    />
  </div>
</aside>

<style>
  .sidebar {
    width: 360px;
    min-width: 360px;
    max-width: 360px;
    flex: 0 0 360px;

    height: 100%;
    max-height: 100%;
    background: #161618;
    border: 1px solid #2a2a2e;
    border-radius: 12px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
  }

  .sidebar__header {
    padding: 1.1rem 1.25rem 1rem;
    background: #1c1c1f;
    border-bottom: 1px solid #2a2a2e;
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

  .engine-controls {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.65rem 1.1rem;
    background: #1c1c1f;
    border-bottom: 1px solid #2a2a2e;
    flex-shrink: 0;
  }

  .engine-controls__left {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: #888;
  }

  .engine-controls__label {
    font-family: 'Outfit', sans-serif;
    font-size: 0.85rem;
    font-weight: 600;
    color: #888;
    letter-spacing: 0.3px;
  }

  .engine-controls__right {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .engine-status {
    font-family: 'Outfit', sans-serif;
    font-size: 0.75rem;
    font-weight: 500;
    border-radius: 8px;
    color: #888;
    padding: 2px 8px;
    transition: all 0.2s ease;
  }

  .engine-toggle {
    width: 36px;
    height: 20px;
    border-radius: 10px;
    border: none;
    background: #333;
    cursor: pointer;
    position: relative;
    transition: background 0.2s ease;
    flex-shrink: 0;
  }

  .engine-toggle--on {
    background: #2b5743;
  }

  .engine-toggle__knob {
    position: absolute;
    top: 3px;
    left: 3px;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: #666;
    transition:
      transform 0.2s ease,
      background 0.2s ease;
  }

  .engine-toggle--on .engine-toggle__knob {
    transform: translateX(16px);
    background: #8be1b4;
  }

  .sidebar__controls {
    padding: 0.75rem 1rem;
    background: #1c1c1f;
    border-top: 1px solid #2a2a2e;
    flex-shrink: 0;
    margin-top: auto;
  }
</style>
