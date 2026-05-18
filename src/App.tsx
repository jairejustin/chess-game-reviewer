import "./App.css";
import { invoke } from "@tauri-apps/api/core";
import { StandardChessboard } from "./components/Board";
import { useGameStore } from "./store/useGameStore";

function App() {
  const { moves, activePly, setActivePly } = useGameStore();
  const currentFen = moves[activePly]?.fen ?? "start";
  const canGoBack = activePly > 0;
  const canGoForward = activePly < moves.length - 1;

  return (
    <main>
      <StandardChessboard
        fen={currentFen}
        config={{ viewOnly: true }}
      />
      <button
        onClick={() => setActivePly(activePly - 1)}
        disabled={!canGoBack}
      >
        {"<"}
      </button>
      <button
        onClick={() => setActivePly(activePly + 1)}
        disabled={!canGoForward}
      >
        {">"}
      </button>
      <button onClick={async () => {
          const pgn = `[Event "Test"]
        [White "Player1"]
        [Black "Player2"]
        [Result "1-0"]

        1. e4 e5 2. Nf3 Nc6 3. Bc4 Bc5 4. b4 Bxb4 5. c3 Ba5 6. d4 exd4 7. O-O d3 8. Qb3 Qf6 9. e5 Qg6 10. Re1 Nge7 11. Ba3 b5 12. Qxb5 Rb8 13. Qa4 Bb6 14. Nbd2 Bb7 15. Ne4 Qf5 16. Bxd3 Qh5 17. Nf6+ gxf6 18. exf6 Rg8 19. Rad1 Qxf3 20. Rxe7+ Nxe7 21. Qxd7+ Kxd7 22. Bf5+ Ke8 23. Bd7+ Kf8 24. Bxe7# 1-0`;

          const result = await invoke("analyze_game", { pgn });
          console.log(result);
        }}>
          Run Full Analysis
      </button>
    </main>
  );
}

export default App;