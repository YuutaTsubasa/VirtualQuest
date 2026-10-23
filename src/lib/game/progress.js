// 闖關進度（存在這台電腦）：打到過第幾大關，下次就能從那一大關開始
const KEY = 'vq_world_reached';

/** 可以選的最後一大關（0 起算） */
export function reachedWorld() {
  try {
    return Math.min(5, Math.max(0, Number(localStorage.getItem(KEY)) || 0));
  } catch {
    return 0;
  }
}

/** 到達第 world 大關（0 起算） */
export function reachWorld(world) {
  try {
    if (world > reachedWorld()) localStorage.setItem(KEY, String(Math.min(5, world)));
  } catch {
    // 不能存就算了
  }
}
