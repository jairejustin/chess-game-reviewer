export function calculateMaterial(fenString: string) {
  const fen =
    fenString === 'start' || !fenString
      ? 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1'
      : fenString;

  const startingCounts: Record<string, number> = {
    P: 8,
    N: 2,
    B: 2,
    R: 2,
    Q: 1,
    p: 8,
    n: 2,
    b: 2,
    r: 2,
    q: 1
  };

  const currentCounts: Record<string, number> = {
    P: 0,
    N: 0,
    B: 0,
    R: 0,
    Q: 0,
    p: 0,
    n: 0,
    b: 0,
    r: 0,
    q: 0
  };

  const boardPart = fen.split(' ')[0];
  for (const char of boardPart) {
    if (currentCounts[char] !== undefined) {
      currentCounts[char]++;
    }
  }

  const blackCaptured: string[] = [];
  const whiteCaptured: string[] = [];

  const values: Record<string, number> = { p: 1, n: 3, b: 3, r: 5, q: 9 };

  let whiteScore = 0;
  let blackScore = 0;

  const whitePieceKeys = ['P', 'N', 'B', 'R', 'Q'];
  for (const key of whitePieceKeys) {
    const lost = startingCounts[key] - currentCounts[key];
    const lowerKey = key.toLowerCase();
    for (let i = 0; i < lost; i++) {
      blackCaptured.push(lowerKey);
      blackScore += values[lowerKey] || 0;
    }
  }

  const blackPieceKeys = ['p', 'n', 'b', 'r', 'q'];
  for (const key of blackPieceKeys) {
    const lost = startingCounts[key] - currentCounts[key];
    for (let i = 0; i < lost; i++) {
      whiteCaptured.push(key);
      whiteScore += values[key] || 0;
    }
  }

  const whiteAdvantage = whiteScore > blackScore ? whiteScore - blackScore : 0;
  const blackAdvantage = blackScore > whiteScore ? blackScore - whiteScore : 0;

  return {
    whiteCaptured,
    blackCaptured,
    whiteAdvantage,
    blackAdvantage
  };
}
