<script lang="ts">
  import { onMount } from 'svelte';
  import { chessground } from '../actions/chessground';
  import {
    moves,
    activePly,
    currentFen,
    initTauriListeners
  } from '../store/gameStore';
  import { getInBoardBadge, badgeColors } from '../utils/boardBadges';

  import EvalBar from '../components/EvalBar.svelte';
  import BoardControls from '../components/BoardControls.svelte';
  import EngineFeedback from '../components/EngineFeedback.svelte';
  import MoveList from '../components/MoveList.svelte';
  import GameSummary from '../components/GameSummary.svelte';

  import '@lichess-org/chessground/assets/chessground.base.css';
  import '@lichess-org/chessground/assets/chessground.brown.css';
  import '@lichess-org/chessground/assets/chessground.cburnett.css';
  // @ts-ignore
  import '@fontsource/bebas-neue';
  // @ts-ignore
  import '@fontsource-variable/outfit';

  type SidebarView = 'game' | 'summary';
  let sidebarView: SidebarView = 'game';

  let destHighlight = 'rgba(155, 199, 0, 0.41)';

  let cgConfig: any = { fen: 'start', viewOnly: true };

  $: {
    const move = $moves[$activePly];
    let autoShapes = [];
    let lastMove: string[] = [];
    destHighlight = 'rgba(155, 199, 0, 0.41)'; // Reset on ply change

    if (
      move &&
      move.ply > 0 &&
      typeof move.uci === 'string' &&
      move.uci.length >= 4
    ) {
      const orig = move.uci.substring(0, 2);
      const dest = move.uci.substring(2, 4);
      lastMove = [orig, dest];

      if (move.classification) {
        destHighlight = badgeColors[move.classification] + '66';

        autoShapes.push({
          orig: dest,
          brush: 'invisible',
          customSvg: { html: getInBoardBadge(move.classification) }
        });
      }
    }

    cgConfig = {
      fen: $currentFen || 'start',
      viewOnly: true,
      lastMove,
      drawable: {
        brushes: {
          invisible: {
            key: 'i',
            color: 'transparent',
            opacity: 0,
            lineWidth: 1
          }
        },
        autoShapes,
        visible: true
      }
    };
  }

  onMount(() => {
    initTauriListeners().catch(console.error);
  });
</script>

<main class="layout">
  <section class="layout__board">
    <div class="board-row">
      <EvalBar />
      <div
        class="board"
        style="--move-highlight: {destHighlight};"
        use:chessground={cgConfig}
      ></div>
    </div>
    <BoardControls />
  </section>

  <aside class="sidebar">
    <div class="sidebar__header">
      <h2 class="sidebar__title">Game Analysis</h2>
      <span class="sidebar__ply-count">{$moves.length} plies</span>
    </div>

    <div class="sidebar__nav">
      <button
        class="sidebar__nav-btn"
        class:sidebar__nav-btn--active={sidebarView === 'game'}
        on:click={() => (sidebarView = 'game')}
      >
        Game
      </button>
      <button
        class="sidebar__nav-btn"
        class:sidebar__nav-btn--active={sidebarView === 'summary'}
        on:click={() => (sidebarView = 'summary')}
      >
        Summary
      </button>
    </div>

    {#if sidebarView === 'game'}
      <EngineFeedback />
      <MoveList />
    {:else if sidebarView === 'summary'}
      <GameSummary />
    {/if}
  </aside>
</main>

<style>
  /* ── Global ──────────────────────────────────────────────────────── */
  :global(body) {
    background-color: #0f0f11;
    color: #ececec;
    font-family: 'Outfit', system-ui, sans-serif;
    margin: 0;
  }

  :global(.cg-wrap square.last-move) {
    background-color: var(--move-highlight) !important;
  }

  :global(.badge-anim) {
    animation: badge-pop-in 0.25s cubic-bezier(0.175, 0.885, 0.32, 1.275) 0.10s both;
  }

  @keyframes badge-pop-in {
    0% { 
      opacity: 0; 
      transform: scale(0.5); 
    }
    100% { 
      opacity: 1; 
      transform: scale(1); 
    }
  }

  /* ── Layout ──────────────────────────────────────────────────────── */
  .layout {
    display: flex;
    height: 100vh;
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem;
    gap: 2rem;
    box-sizing: border-box;
  }
  .layout__board {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 1.5rem;
    min-width: 0;
  }

  /* ── Board row ───────────────────────────────────────────────────── */
  .board-row {
    display: flex;
    flex-direction: row;
    align-items: stretch;
    gap: 12px;
    width: 100%;
    max-width: 680px;
  }
  .board {
    flex: 1;
    aspect-ratio: 1 / 1;
    border-radius: 6px;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.6);
    min-width: 0;
  }

  /* ── Sidebar Framework ───────────────────────────────────────────── */
  .sidebar {
    width: 360px;
    flex-shrink: 0;
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
  .sidebar__ply-count {
    font-size: 0.8rem;
    color: #555;
    font-weight: 600;
  }

  /* ── Nav switcher ────────────────────────────────────────────────── */
  .sidebar__nav {
    display: flex;
    flex-shrink: 0;
    border-bottom: 1px solid #2a2a2e;
    background: #1c1c1f;
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
</style>
