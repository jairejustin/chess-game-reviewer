import { useEffect, useRef } from "react";
import { Chessground } from "@lichess-org/chessground";
import { Config } from "@lichess-org/chessground/config";
import { Api } from "@lichess-org/chessground/api";
import "./Board.css";
import "@lichess-org/chessground/assets/chessground.base.css";
import "@lichess-org/chessground/assets/chessground.brown.css";
import "@lichess-org/chessground/assets/chessground.cburnett.css";

interface ChessboardProps {
  fen?: string;
  orientation?: "white" | "black";
  onMove?: (orig: string, dest: string) => void;
  config?: Partial<Config>;
}

export const StandardChessboard = ({
  fen = "start",
  orientation = "white",
  onMove,
  config = {},
}: ChessboardProps) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const cgRef = useRef<Api | null>(null);
  const onMoveRef = useRef(onMove);

  useEffect(() => { onMoveRef.current = onMove; }, [onMove]);

  useEffect(() => {
    if (!containerRef.current) return;
    cgRef.current = Chessground(containerRef.current, {
      ...config,
      fen,
      orientation,
      movable: {
        color: orientation,
        free: false,
        events: {
          after: (orig, dest) => onMoveRef.current?.(orig, dest),
        },
      },
    });
    return () => {
      cgRef.current?.destroy();
      cgRef.current = null;
    };
  }, []);

  useEffect(() => {
    if (!cgRef.current) return;
    cgRef.current.set({ fen, orientation });
  }, [fen, orientation]);

  return (
    <div
      ref={containerRef}
      className="boardContainer"
    />
  );
};