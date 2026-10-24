import { test } from 'node:test';
import assert from 'node:assert/strict';
import * as combo from '../src/lib/game/combo.js';

// 模擬一格一格的遊戲事件
const step = (frame, events, players = 2) => combo.onEvents({ frame, events, players: Array.from({ length: players }, (_, slot) => ({ slot })) });

test('combo counts hits within the gap and resets when hurt or too slow', () => {
  step(0, [{ type: 'stage' }]);
  step(10, [{ type: 'hit', by: 0 }]);
  step(30, [{ type: 'hit', by: 0 }]);
  assert.equal(combo.comboOf(0, 30), 0, '兩下還不顯示');
  step(50, [{ type: 'hit', by: 0 }]);
  assert.equal(combo.comboOf(0, 50), 3);
  step(60, [{ type: 'hurt', slot: 0 }]);
  assert.equal(combo.comboOf(0, 60), 0, '被打就斷');
  step(70, [{ type: 'hit', by: 1 }]);
  step(200, [{ type: 'hit', by: 1 }]);
  step(210, [{ type: 'hit', by: 1 }]);
  assert.equal(combo.comboOf(1, 210), 0, '隔太久重新算');
});

test('stage rank: fast and clean is S, slow and battered is C', () => {
  step(0, [{ type: 'stage' }]);
  for (let f = 10; f < 10 + 15 * 20; f += 20) step(f, [{ type: 'hit', by: 0 }]);
  step(60 * 70, [{ type: 'clear', world: 9, sub: 0 }]);
  assert.equal(combo.result.rank, 'S');
  assert.equal(combo.result.maxCombo, 15);

  step(0, [{ type: 'stage' }]);
  for (let k = 0; k < 20; k++) step(10 + k, [{ type: 'hurt', slot: k % 2 }]);
  step(60 * 220, [{ type: 'clear', world: 9, sub: 1 }]);
  assert.equal(combo.result.rank, 'C');
  assert.equal(combo.result.seconds, 220);
});
