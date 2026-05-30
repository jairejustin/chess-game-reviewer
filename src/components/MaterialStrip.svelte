<script lang="ts">
  import ChessQueen from 'lucide-svelte/icons/chess-queen';
  import ChessRook from 'lucide-svelte/icons/chess-rook';
  import ChessBishop from 'lucide-svelte/icons/chess-bishop';
  import ChessKnight from 'lucide-svelte/icons/chess-knight';
  import ChessPawn from 'lucide-svelte/icons/chess-pawn';

  export let capturedPieces: string[] = [];
  export let advantage: number = 0;

  const pieceIcons: Record<string, any> = {
    q: ChessQueen,
    r: ChessRook,
    b: ChessBishop,
    n: ChessKnight,
    p: ChessPawn
  };
</script>

<div class="material-strip">
  {#each capturedPieces as piece}
    <svelte:component
      this={pieceIcons[piece]}
      size={16}
      strokeWidth={2.5}
      class="material-strip__icon"
    />
  {/each}

  {#if advantage > 0}
    <span class="material-strip__score">+{advantage}</span>
  {/if}
</div>

<style>
  .material-strip {
    display: flex;
    align-items: center;
    gap: 1px;
    height: 20px;
    font-family: 'Outfit', sans-serif;
  }

  :global(.material-strip__icon) {
    color: #888;
  }

  .material-strip__score {
    font-size: 0.95rem;
    font-weight: 600;
    color: #95bb4a;
    margin-left: 6px;
    transform: translateY(1px);
  }
</style>
