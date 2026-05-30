<script lang="ts">
  import ChessKing from 'lucide-svelte/icons/chess-king';
  import ChessQueen from 'lucide-svelte/icons/chess-queen';
  import ChessRook from 'lucide-svelte/icons/chess-rook';
  import ChessBishop from 'lucide-svelte/icons/chess-bishop';
  import ChessKnight from 'lucide-svelte/icons/chess-knight';

  export let san: string;

  const pieceIcons: Record<string, any> = {
    K: ChessKing,
    Q: ChessQueen,
    R: ChessRook,
    B: ChessBishop,
    N: ChessKnight
  };

  $: parsed = (() => {
    if (san === 'O-O' || san === 'O-O-O') return { piece: null, text: san };

    const match = san.match(/^([KQRBN])?(.*)$/);

    if (match && match[1]) {
      return { piece: match[1], text: match[2] };
    }
    return { piece: null, text: san };
  })();
</script>

<span class="figurine-move">
  {#if parsed.piece}
    <span class="figurine-move__icon">
      <svelte:component
        this={pieceIcons[parsed.piece]}
        size={20}
        strokeWidth={1.5}
      />
    </span>
  {/if}
  <span class="figurine-move__text">{parsed.text}</span>
</span>

<style>
  .figurine-move {
    display: inline-flex;
    align-items: center;
    font-family: 'Outfit', sans-serif;
    font-weight: 600;
    line-height: 1rem;
  }

  .figurine-move__icon {
    display: inline-flex;
    align-items: center;
    transform: translateY(-1.5px);
  }

  .figurine-move__text {
    letter-spacing: 0.2px;
  }
</style>
