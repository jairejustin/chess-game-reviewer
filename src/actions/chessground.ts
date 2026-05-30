import { Chessground } from '@lichess-org/chessground';
import type { Config } from '@lichess-org/chessground/config';
import type { Api } from '@lichess-org/chessground/api';

export function chessground(node: HTMLElement, config: Config) {
  let cg: Api = Chessground(node, config);

  return {
    update(newConfig: Config) {
      cg.set(newConfig);
    },
    destroy() {
      cg.destroy();
    }
  };
}
