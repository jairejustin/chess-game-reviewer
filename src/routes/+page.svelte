<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from "@tauri-apps/api/core";
  import { chessground } from "../actions/chessground";
  import {
    moves,
    activePly,
    currentFen,
    currentEval,
    isAnalyzing,
    analysisSummary,
    initTauriListeners
  } from "../store/gameStore";

  import "@lichess-org/chessground/assets/chessground.base.css";
  import "@lichess-org/chessground/assets/chessground.brown.css";
  import "@lichess-org/chessground/assets/chessground.cburnett.css";


  $: canGoBack = $activePly > 0;
  $: canGoForward = $activePly < $moves.length - 1;

  function goBack() { if (canGoBack) activePly.update(p => p - 1); }
  function goForward() { if (canGoForward) activePly.update(p => p + 1); }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowLeft') goBack();
    if (e.key === 'ArrowRight') goForward();
  }


  function winPercent(cp: number): number {
    return 50 + 50 * (2 / (1 + Math.exp(-0.00368 * cp)) - 1);
  }

  $: whitePercent = winPercent($currentEval);
  $: blackPercent = 100 - whitePercent;


  function formatEval(cp: number): string {
    if (cp >= 10000) return '+M';
    if (cp <= -10000) return '-M';
    const abs = Math.abs(cp / 100).toFixed(2);
    return cp >= 0 ? `+${abs}` : `-${abs}`;
  }

  function formatAccuracy(score: number): string {
    return score.toFixed(1) + '%';
  }

  const badgeColors: Record<string, string> = {
    brilliant:  '#1baca6',
    great:      '#5c8bb0',
    best:       '#95bb4a',
    excellent:  '#96bc4b',
    good:       '#96bc4b',
    inaccuracy: '#f4c153',
    mistake:    '#e6912c',
    blunder:    '#b33430',
    miss:       '#ff7769',
    book:       '#a88865',
    forced:     '#8da6b6'
  };

  const badgeIcons: Record<string, string> = {
    brilliant:  '!!',
    great:      '!',
    best:       '★',
    excellent:  '✓',
    good:       '·',
    inaccuracy: '?!',
    mistake:    '?',
    blunder:    '??',
    miss:       '✗',
    book:       '📖',
    forced:     '→'
  };


  onMount(() => {
    initTauriListeners().catch(console.error);
  });

  async function runAnalysis() {
    const pgn = `[Event "Test"]\n[White "Player1"]\n[Black "Player2"]\n[Result "1-0"]\n\n1. e4 e5 2. Nf3 Nc6 3. Bc4 Bc5 4. b4 Bxb4 5. c3 Ba5 6. d4 exd4 7. O-O d3 8. Qb3 Qf6 9. e5 Qg6 10. Re1 Nge7 11. Ba3 b5 12. Qxb5 Rb8 13. Qa4 Bb6 14. Nbd2 Bb7 15. Ne4 Qf5 16. Bxd3 Qh5 17. Nf6+ gxf6 18. exf6 Rg8 19. Rad1 Qxf3 20. Rxe7+ Nxe7 21. Qxd7+ Kxd7 22. Bf5+ Ke8 23. Bd7+ Kf8 24. Bxe7# 1-0`;

    try {
      await invoke("analyze_game", { pgn });
    } catch (e) {
      console.error("Analysis failed:", e);
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<main class="app-layout">

  <section class="board-section">
    <div class="board-row">

      <div class="eval-bar-wrapper" aria-label="Evaluation bar, {formatEval($currentEval)}">
        <div class="eval-bar">
          <div
            class="eval-segment black-segment"
            style="flex: {blackPercent}"
          ></div>

          <div class="eval-divider"></div>

          <div
            class="eval-segment white-segment"
            style="flex: {whitePercent}"
          ></div>
        </div>

        <span class="eval-label" class:winning={$currentEval > 0} class:losing={$currentEval < 0}>
          {formatEval($currentEval)}
        </span>
      </div>

      <div
        class="boardContainer"
        use:chessground={{ fen: $currentFen, viewOnly: true }}
      ></div>
    </div>

    <div class="controls">
      <button on:click={goBack} disabled={!canGoBack} title="Previous move (←)">‹</button>
      <button on:click={goForward} disabled={!canGoForward} title="Next move (→)">›</button>
      <button class="analyze-btn" on:click={runAnalysis} disabled={$isAnalyzing}>
        {#if $isAnalyzing}
          <span class="spinner"></span> Analyzing…
        {:else}
          Run Analysis
        {/if}
      </button>
    </div>
  </section>

  <aside class="sidebar">
    <div class="sidebar-header">
      <h2>Game Analysis</h2>
      <span class="move-count">{$moves.length} plies</span>
    </div>

    {#if $analysisSummary}
      {@const s = $analysisSummary}
      <div class="accuracy-panel">
        <div class="accuracy-row">
          <div class="player-label white-player">
            <span class="player-dot white-dot"></span>
            {s.metadata.white}
          </div>
          <div class="accuracy-bar-wrapper">
            <div class="accuracy-bar" style="width: {s.whiteAccuracy}%"></div>
          </div>
          <span class="accuracy-score">{formatAccuracy(s.whiteAccuracy)}</span>
        </div>
        <div class="accuracy-row">
          <div class="player-label black-player">
            <span class="player-dot black-dot"></span>
            {s.metadata.black}
          </div>
          <div class="accuracy-bar-wrapper">
            <div class="accuracy-bar black-bar" style="width: {s.blackAccuracy}%"></div>
          </div>
          <span class="accuracy-score">{formatAccuracy(s.blackAccuracy)}</span>
        </div>

        <div class="counts-row">
          {#each Object.entries(s.moveCounts) as [badge, count]}
            {#if count > 0}
              <span
                class="count-chip"
                style="background: {badgeColors[badge] ?? '#555'}"
                title="{badge}: {count}"
              >
                {badgeIcons[badge] ?? badge} {count}
              </span>
            {/if}
          {/each}
        </div>
      </div>
    {/if}

    <div class="moves-list">
      {#if $moves.length === 0}
        <div class="empty-state">
          {$isAnalyzing ? 'Engine is thinking…' : 'No moves analyzed yet.'}
        </div>
      {:else}
        {#each $moves as move, index}
          <button
            class="move-row"
            class:active={$activePly === index}
            on:click={() => activePly.set(index)}
            aria-label="Go to {Math.ceil(move.ply / 2)}{move.ply % 2 !== 0 ? '.' : '...'} {move.san}"
          >
            <div class="move-info">
              <span class="ply-number">
                {Math.ceil(move.ply / 2)}{move.ply % 2 !== 0 ? '.' : '...'}
              </span>
              <span class="san">{move.san}</span>
            </div>

            <div class="eval-info">
              <span class="eval-score">{formatEval(move.playedEval)}</span>
              {#if move.classification === 'book'}
                <span class="badge" style="background-color: {badgeColors.book}">
                  {badgeIcons.book}
                </span>
              {:else}
                <span 
                  class="badge" 
                  style="background-color: {badgeColors[move.classification] ?? '#555'}" 
                  title={move.classification}
                >
                  {badgeIcons[move.classification] ?? move.classification}
                </span>
              {/if}
            </div>
          </button>
        {/each}
      {/if}
    </div>
  </aside>
</main>

<style>
  :global(body) {
    background-color: #121212;
    color: #ececec;
    font-family: system-ui, -apple-system, sans-serif;
    margin: 0;
  }

  .app-layout {
    display: flex;
    height: 100vh;
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem;
    gap: 2rem;
    box-sizing: border-box;
  }

  .board-section {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 1.5rem;
    min-width: 0;
  }

  .board-row {
    display: flex;
    flex-direction: row;
    align-items: stretch;
    gap: 8px;
    width: 100%;
    max-width: 680px;
  }

  .boardContainer {
    flex: 1;
    aspect-ratio: 1 / 1;
    border-radius: 4px;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
    min-width: 0;
  }

  .eval-bar-wrapper {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
  }

  .eval-bar {
    width: 18px;
    flex: 1;
    border-radius: 6px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    background: #1a1a1a;
    box-shadow: inset 0 0 0 1px rgba(255,255,255,0.06);
  }

  .eval-segment {
    /* flex value is set inline via the whitePercent / blackPercent reactives.
       transition gives the smooth animation as eval changes between plies. */
    transition: flex 0.35s cubic-bezier(0.4, 0, 0.2, 1);
    min-height: 0;
  }

  .black-segment {
    background: #1e1e1e;
  }

  .white-segment {
    background: #f0ede8;
  }

  .eval-divider {
    height: 2px;
    flex-shrink: 0;
    background: rgba(255, 255, 255, 0.15);
  }

  .eval-label {
    font-family: monospace;
    font-size: 0.7rem;
    font-weight: 700;
    color: #888;
    white-space: nowrap;
    letter-spacing: -0.5px;
  }

  .eval-label.winning { color: #95bb4a; }
  .eval-label.losing  { color: #e06060; }

  .controls {
    display: flex;
    gap: 0.75rem;
    align-items: center;
  }

  button {
    background: #2a2a2a;
    color: #fff;
    border: 1px solid #444;
    padding: 0.5rem 1.25rem;
    border-radius: 6px;
    cursor: pointer;
    font-weight: 600;
    font-size: 1rem;
    transition: background 0.15s;
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  button:hover:not(:disabled) { background: #3a3a3a; }

  button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .analyze-btn {
    padding: 0.5rem 1.5rem;
    background: #1d3a2e;
    border-color: #2d6a4e;
    color: #7ec8a0;
  }

  .analyze-btn:hover:not(:disabled) { background: #254d3c; }

  .spinner {
    display: inline-block;
    width: 12px;
    height: 12px;
    border: 2px solid rgba(126, 200, 160, 0.3);
    border-top-color: #7ec8a0;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .sidebar {
    width: 340px;
    flex-shrink: 0;
    background: #1a1a1a;
    border: 1px solid #2e2e2e;
    border-radius: 10px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .sidebar-header {
    padding: 1rem 1.25rem;
    background: #202020;
    border-bottom: 1px solid #2e2e2e;
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-shrink: 0;
  }

  .sidebar-header h2 {
    font-size: 1rem;
    font-weight: 700;
    margin: 0;
    letter-spacing: 0.3px;
  }

  .move-count {
    font-size: 0.8rem;
    color: #666;
  }

  .accuracy-panel {
    padding: 1rem 1.25rem;
    border-bottom: 1px solid #2e2e2e;
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
    flex-shrink: 0;
  }

  .accuracy-row {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }

  .player-label {
    font-size: 0.8rem;
    font-weight: 600;
    display: flex;
    align-items: center;
    gap: 0.4rem;
    width: 90px;
    flex-shrink: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .player-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .white-dot { background: #e8e6e0; border: 1px solid #555; }
  .black-dot { background: #2a2a2a; border: 1px solid #555; }

  .accuracy-bar-wrapper {
    flex: 1;
    height: 6px;
    background: #2e2e2e;
    border-radius: 3px;
    overflow: hidden;
  }

  .accuracy-bar {
    height: 100%;
    background: #95bb4a;
    border-radius: 3px;
    transition: width 0.6s ease;
    max-width: 100%;
  }

  .black-bar { background: #5c8bb0; }

  .accuracy-score {
    font-family: monospace;
    font-size: 0.8rem;
    font-weight: 700;
    color: #ccc;
    width: 42px;
    text-align: right;
    flex-shrink: 0;
  }

  .counts-row {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    padding-top: 4px;
  }

  .count-chip {
    font-size: 0.65rem;
    font-weight: 700;
    padding: 2px 6px;
    border-radius: 10px;
    color: #fff;
    text-shadow: 0 1px 2px rgba(0,0,0,0.4);
    letter-spacing: 0.3px;
  }

  .moves-list {
    flex: 1;
    overflow-y: auto;
    padding: 0.4rem;
    scrollbar-width: thin;
    scrollbar-color: #333 transparent;
  }

  .empty-state {
    padding: 2.5rem;
    text-align: center;
    color: #555;
    font-style: italic;
    font-size: 0.9rem;
  }

  .move-row {
    appearance: none;
    background: transparent;
    border: 1px solid transparent;
    text-align: left;
    font-family: inherit;
    color: inherit;
    width: 100%;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.6rem 0.9rem;
    margin-bottom: 2px;
    border-radius: 6px;
    cursor: pointer;
    transition: background 0.1s;
  }

  .move-row:hover { background: #252525; }

  .move-row:focus-visible {
    outline: 2px solid #5c8bb0;
    outline-offset: -2px;
  }

  .move-row.active {
    background: #2b2b2b;
    border-color: #3a3a3a;
    border-left: 3px solid #5c8bb0;
  }

  .move-info {
    display: flex;
    gap: 0.6rem;
    align-items: center;
  }

  .ply-number {
    color: #666;
    font-family: monospace;
    font-size: 0.8rem;
    width: 42px;
    flex-shrink: 0;
  }

  .san {
    font-weight: 600;
    font-size: 1rem;
  }

  .eval-info {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .eval-score {
    font-family: monospace;
    color: #777;
    font-size: 0.82rem;
    width: 52px;
    text-align: right;
  }

  .badge {
    min-width: 22px;
    height: 20px;
    padding: 0 5px;
    border-radius: 10px;
    font-size: 0.72rem;
    font-weight: 700;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #fff;
    text-shadow: 0 1px 2px rgba(0,0,0,0.4);
  }
</style>