<script lang="ts">
  import type { PVLine } from '$lib/types/game';
  import { enterVariationFromPV } from '$lib/stores/explorerStore';
  import Figurine from '$lib/components/ui/Figurine.svelte';

  export let lines: PVLine[] = [];

  export let active: boolean = true;
  export let depth: number = 0;

  export let status: 'thinking' | 'paused' | 'starting' = 'thinking';

  $: isTerminal = status === 'paused' && lines.length === 0;
  $: isLoading = lines.length === 0 && !isTerminal;

  function handleLineClick(line: PVLine) {
    if (line.uciMoves.length === 0) return;
    enterVariationFromPV(line);
  }

  function evalClass(evalStr: string): string {
    if (evalStr.startsWith('-M')) return 'eval--mate-black';
    if (evalStr.includes('M')) return 'eval--mate-white';

    const n = parseFloat(evalStr);
    if (isNaN(n)) return '';
    if (n >= 1.5) return 'eval--winning';
    if (n <= -1.5) return 'eval--losing';

    return 'eval--equal';
  }
</script>

<div class="multipv">
  <div class="multipv__header">
    <span class="multipv__label">Engine Lines</span>
    
    {#if active && (status === 'thinking' || status === 'starting')}
      <span class="multipv__depth">
        DEPTH {depth}
      </span>
    {/if}
  </div>
  

  <div class="multipv__lines">
    {#if isTerminal}
      <div class="multipv__terminal">
        <span class="multipv__terminal-text">No Legal Moves</span>
      </div>
    {:else if isLoading}
      {#each [1, 2, 3] as _}
        <div class="multipv__line multipv__line--skeleton">
          <div class="skeleton__eval"></div>
          <div class="skeleton__moves"></div>
        </div>
      {/each}
    {:else}
      {#each lines as line (line.index)}
        <button
          class="multipv__line"
          class:multipv__line--best={line.index === 1}
          on:click={() => handleLineClick(line)}
          disabled={!active || line.sanMoves.length === 0}
          title="Load this line into the explorer"
        >
          <span class="multipv__eval {evalClass(line.evaluation)}">
            {line.evaluation}
          </span>
          <span class="multipv__moves">
            {#each line.sanMoves.slice(0, 6) as san}
              <span class="multipv__move">
                <Figurine {san} />
              </span>
            {/each}
            {#if line.sanMoves.length > 6}
              <span class="multipv__overflow">+{line.sanMoves.length - 6}</span>
            {/if}
          </span>
        </button>
      {/each}
    {/if}
  </div>
</div>

<style>
  .multipv {
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  .multipv__depth {
    font-family: 'Bebas Neue', sans-serif;
    font-weight: bold;
    letter-spacing: 1px;
    color: #ffffff;
  }

  .multipv__header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.6rem 1rem 0.4rem;
    border-bottom: 1px solid #2a2a2e;
  }

  .multipv__label {
    font-family: 'Bebas Neue', sans-serif;
    font-size: 0.95rem;
    letter-spacing: 1.5px;
    color: #666;
    text-transform: uppercase;
  }

  .multipv__lines {
    display: flex;
    flex-direction: column;
  }

  .multipv__terminal {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.9rem 1rem;
    color: #555;
    font-family: 'Outfit', sans-serif;
    font-size: 0.85rem;
  }

  .multipv__terminal-text {
    font-style: italic;
    letter-spacing: 0.2px;
  }

  .multipv__line {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.55rem 1rem;
    background: transparent;
    border: none;
    border-bottom: 1px solid #1e1e22;
    color: inherit;
    font-family: inherit;
    text-align: left;
    cursor: pointer;
    transition: background 0.12s ease;
    width: 100%;
  }

  .multipv__line:hover:not(:disabled) {
    background: #1e2a22;
  }

  .multipv__line:disabled {
    cursor: default;
  }

  .multipv__line--best {
    background: rgba(139, 225, 180, 0.03);
  }

  .multipv__line--skeleton {
    pointer-events: none;
    cursor: default;
  }

  .multipv__eval {
    font-family: 'Bebas Neue', sans-serif;
    font-size: 1.1rem;
    letter-spacing: 0.5px;
    min-width: 48px;
    flex-shrink: 0;
  }

  .eval--winning {
    color: #95bb4a;
  }
  .eval--losing {
    color: #e06060;
  }
  .eval--equal {
    color: #aaa;
  }
  .eval--mate-white {
    color: #8be1b4;
  }
  .eval--mate-black {
    color: #e06060;
  }

  .multipv__moves {
    display: flex;
    flex-wrap: wrap;
    gap: 2px 6px;
    font-size: 0.9rem;
    color: #bbb;
    font-family: 'Outfit', sans-serif;
    min-width: 0;
    overflow: hidden;
  }

  .multipv__move {
    white-space: nowrap;
  }

  .multipv__overflow {
    font-size: 0.8rem;
    color: #555;
    font-style: italic;
  }

  .skeleton__eval {
    width: 40px;
    height: 18px;
    border-radius: 3px;
    background: linear-gradient(90deg, #222 25%, #2a2a2a 50%, #222 75%);
    background-size: 200% 100%;
    animation: shimmer 1.4s infinite;
  }

  .skeleton__moves {
    flex: 1;
    height: 14px;
    border-radius: 3px;
    background: linear-gradient(90deg, #222 25%, #2a2a2a 50%, #222 75%);
    background-size: 200% 100%;
    animation: shimmer 1.4s infinite 0.2s;
  }

  @keyframes shimmer {
    0% {
      background-position: 200% 0;
    }
    100% {
      background-position: -200% 0;
    }
  }
</style>