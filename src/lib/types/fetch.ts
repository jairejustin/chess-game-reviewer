export type Platform = 'chesscom' | 'lichess';

export interface ChessComCursor {
  archiveIndex: number;
  offset: number;
}

export interface PlayerProfile {
  username: string;
  title: string | null;
  avatarUrl: string | null;
  countryCode: string | null;
  joined: number | null;
  lastOnline: number | null;
  followers: number | null;
  isStreamer: boolean | null;
  twitchUrl: string | null;
  fide: number | null;
}

export interface GamePlayer {
  username: string;
  rating: number | null;
  result: string;
}

export interface GameSummary {
  id: string;
  pgn: string;
  platform: Platform;
  timeClass: string;
  timeControl: string | null;
  playedAt: number;
  rated: boolean;
  white: GamePlayer;
  black: GamePlayer;
}

export interface FetchResult {
  profile: PlayerProfile;
  games: GameSummary[];
  cursor: ChessComCursor | null;
}
