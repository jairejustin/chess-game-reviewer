import { invoke } from '@tauri-apps/api/core';
import type {
  FetchResult,
  ChessComCursor,
  Platform,
  PlayerProfile
} from '../types/fetch';

export const ApiService = {
  async fetchGames(
    username: string,
    platform: Platform,
    cursor: ChessComCursor | null
  ): Promise<FetchResult> {
    return await invoke<FetchResult>('fetch_games', {
      username,
      platform,
      cursor
    });
  },

  async getPlayerProfile(username: string): Promise<PlayerProfile> {
    return await invoke<PlayerProfile>('get_player_profile', { username });
  },

  async parsePgn(pgn: string): Promise<any[]> {
    return await invoke<any[]>('parse_pgn', { pgn });
  }
};
