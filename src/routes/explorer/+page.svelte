<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { get } from 'svelte/store';
  import { Chess } from 'chess.js';
  import Cpu from 'lucide-svelte/icons/cpu';

  import { selectedGame } from '$lib/stores/fetchStore';
  import { moves, activePly } from '$lib/stores/boardStore';
  import {
    currentLine,
    explorerIndex,
    setExplorerIndex,
    activeExplorerFen,
    liveEval,
    liveMateIn,
    engineOn,
    engineStatus,
    toggleEngine,
    currentDepth,
    mountExplorer,
    unmountExplorer,
    livePVLines,
    enterVariationFromMove
  } from '$lib/stores/explorerStore';

  import AppLayout from '$lib/components/ui/AppLayout.svelte';
  import ChessBoard from '$lib/components/board/ChessBoard.svelte';
  import TabbedSidebar from '$lib/components/ui/TabbedSidebar.svelte';
  import MultiPVStack from '$lib/components/explorer/MultiPVStack.svelte';
  import MoveList from '$lib/components/ui/MoveList.svelte';
  import NavigationControls from '$lib/components/ui/NavigationControls.svelte';
  import EngineSettings from '$lib/components/ui/EngineSettings.svelte';

  const explorerTabs = [
    { id: 'live', label: 'Live' },
    { id: 'engine', label: 'Engine' }
  ];
  let activeTab = 'live';

  $: whiteName = $selectedGame?.white.username ?? 'White';
  $: blackName = $selectedGame?.black.username ?? 'Black';

  $: legalDests = computeLegalDests($activeExplorerFen);

  $: boardEngineLines = $livePVLines
    .filter((line) => line.uciMoves && line.uciMoves.length > 0)
    .map((line, i) => {
      const firstUci = line.uciMoves[0];
      return {
        orig: firstUci.substring(0, 2),
        dest: firstUci.substring(2, 4),
        rank: i + 1
      };
    });

  function computeLegalDests(fen: string): Map<string, string[]> {
    const dests = new Map<string, string[]>();
    try {
      const chess = new Chess(fen === 'start' ? undefined : fen);
      const chessMoves = chess.moves({ verbose: true });
      for (const move of chessMoves) {
        const existing = dests.get(move.from) ?? [];
        existing.push(move.to);
        dests.set(move.from, existing);
      }
    } catch {}
    return dests;
  }

  function handleBoardMove(orig: string, dest: string) {
    const fen = get(activeExplorerFen);
    try {
      const chess = new Chess(fen === 'start' ? undefined : fen);
      const result = chess.move({ from: orig, to: dest, promotion: 'q' });
      if (!result) return;
      enterVariationFromMove(
        result.san,
        chess.fen(),
        orig + dest + (result.promotion ?? '')
      );
    } catch {}
  }

  onMount(async () => {
    await mountExplorer(get(moves), get(activePly));
  });

  onDestroy(async () => {
    await unmountExplorer();
  });
</script>

<AppLayout>
  <svelte:fragment slot="board">
    <ChessBoard
      {whiteName}
      {blackName}
      whiteRating={$selectedGame?.white.rating ?? null}
      blackRating={$selectedGame?.black.rating ?? null}
      evalCp={$liveEval}
      evalMateIn={$liveMateIn}
      evalActive={$engineOn}
      fen={$activeExplorerFen}
      currentMove={$currentLine[$explorerIndex]}
      viewOnly={false}
      {legalDests}
      onMove={handleBoardMove}
      engineLines={boardEngineLines}
    />
  </svelte:fragment>

  <svelte:fragment slot="sidebar">
    <TabbedSidebar title="Live Explorer" tabs={explorerTabs} bind:activeTab>
      {#if activeTab === 'live'}
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
                Starting
              {:else}
                Paused
              {/if}
            </span>
            <button
              class="engine-toggle"
              class:engine-toggle--on={$engineOn}
              on:click={() => toggleEngine(!$engineOn)}
              aria-label={$engineOn
                ? 'Turn Stockfish off'
                : 'Turn Stockfish on'}
              title={$engineOn ? 'Turn Stockfish off' : 'Turn Stockfish on'}
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
      {:else if activeTab === 'engine'}
        <EngineSettings />
      {/if}
    </TabbedSidebar>
  </svelte:fragment>
</AppLayout>

<style>
  .engine-controls {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.65rem 1.1rem;
    background: var(--bg-surface, #1c1c1f);
    border-bottom: 1px solid var(--border-subtle, #2a2a2e);
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
    background: var(--bg-surface, #1c1c1f);
    border-top: 1px solid var(--border-subtle, #2a2a2e);
    flex-shrink: 0;
    margin-top: auto;
  }
</style>
