<script>
  // 右下角的操作提示：依目前用的是鍵盤、手把還是觸控換說法
  import { asset } from '../art-style.svelte.js';
  import { t } from '../i18n.svelte.js';
  import { ui } from '../ui.svelte.js';

  let { extra = '' } = $props();
  const icon = $derived(ui.input === 'pad' ? 'gamepad' : ui.input === 'touch' ? 'touch' : 'keyboard');
  const text = $derived(
    ui.input === 'pad'
      ? t('十字鍵 選擇　A 確定　B 返回')
      : ui.input === 'touch'
        ? t('點選操作')
        : t('↑↓←→ 選擇　Enter 確定　Esc 返回　M 靜音'),
  );
</script>

<div class="hint-bar control-hint">
  <img src="{asset(`ui/icons/${icon}.webp`)}" alt="" />
  <span>{extra || text}</span>
</div>

<style>
  .control-hint {
    justify-self: end;
    font-size: 2.1cqh;
  }
  img {
    width: 4.2cqh;
    height: 4.2cqh;
  }
</style>
