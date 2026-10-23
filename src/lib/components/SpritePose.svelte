<script>
  // 從角色圖集裡取一個動作畫出來（選單預覽用）
  const POSES = ['idle', 'run1', 'run2', 'jump', 'spike', 'dive', 'win', 'lose'];

  let { character, pose = 'idle', flip = false, size = '30cqh' } = $props();
  const index = $derived(Math.max(0, POSES.indexOf(pose)));
</script>

<div
  class="pose"
  class:flip
  role="img"
  aria-label={character.name}
  style="--size: {size}; background-image: url({character.spriteSrc}); background-position: {(index / (POSES.length - 1)) * 100}% 0"
></div>

<style>
  .pose {
    width: var(--size);
    height: var(--size);
    background-size: 800% 100%;
    background-repeat: no-repeat;
  }
  .flip {
    transform: scaleX(-1);
  }
</style>
