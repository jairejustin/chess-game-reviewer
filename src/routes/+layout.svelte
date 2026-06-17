<script>
  import '@lichess-org/chessground/assets/chessground.base.css';
  import '@lichess-org/chessground/assets/chessground.brown.css';
  // @ts-ignore
  import '@fontsource/bebas-neue';
  // @ts-ignore
  import '@fontsource-variable/outfit';
</script>

<slot />

<style>
  :global(body) {
    background-color: #0f0f11;
    color: #ececec;
    font-family: 'Outfit', system-ui, sans-serif;
    margin: 0;
  }

  /* Chessground Overrides */
  :global(.cg-wrap svg.cg-shapes) {
    z-index: 1 !important;
  }

  :global(.cg-wrap piece) {
    z-index: 2 !important;
  }
  
  :global(.cg-wrap piece.dragging) {
    z-index: 10 !important;
  }

  :global(.cg-wrap svg.cg-custom-svgs) {
    z-index: 3 !important;
    overflow: visible !important;
  }

  /* --- MAESTRO PIECE SET --- */
  :global(.cg-wrap piece.pawn.white) {
    background-image: url('/assets/pieces/maestro/wP.svg');
  }
  :global(.cg-wrap piece.knight.white) {
    background-image: url('/assets/pieces/maestro/wN.svg');
  }
  :global(.cg-wrap piece.bishop.white) {
    background-image: url('/assets/pieces/maestro/wB.svg');
  }
  :global(.cg-wrap piece.rook.white) {
    background-image: url('/assets/pieces/maestro/wR.svg');
  }
  :global(.cg-wrap piece.queen.white) {
    background-image: url('/assets/pieces/maestro/wQ.svg');
  }
  :global(.cg-wrap piece.king.white) {
    background-image: url('/assets/pieces/maestro/wK.svg');
  }

  :global(.cg-wrap piece.pawn.black) {
    background-image: url('/assets/pieces/maestro/bP.svg');
  }
  :global(.cg-wrap piece.knight.black) {
    background-image: url('/assets/pieces/maestro/bN.svg');
  }
  :global(.cg-wrap piece.bishop.black) {
    background-image: url('/assets/pieces/maestro/bB.svg');
  }
  :global(.cg-wrap piece.rook.black) {
    background-image: url('/assets/pieces/maestro/bR.svg');
  }
  :global(.cg-wrap piece.queen.black) {
    background-image: url('/assets/pieces/maestro/bQ.svg');
  }
  :global(.cg-wrap piece.king.black) {
    background-image: url('/assets/pieces/maestro/bK.svg');
  }

  :global(.cg-wrap square.last-move) {
    background-color: var(--move-highlight) !important;
  }

  :global(.badge-anim) {
    animation: badge-pop-in 0.1s cubic-bezier(0.175, 0.885, 0.32, 1.275) 0.1s
      both;
  }

  :global(.spin) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
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
</style>
