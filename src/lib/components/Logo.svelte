<script>
  // 「虛擬闖天下」標誌（art-work/ui-concept-v1/logo-transparent.png 裁掉透明邊，ui/logo.webp）
  import { asset, artStyle } from '../art-style.svelte.js';
  import { hasAsset } from '../style-assets.js';
  import { t } from '../i18n.svelte.js';
  let { size = 'small' } = $props();
</script>

<!-- own：這個畫風有自己的 Logo（例如 Game Boy 的四色版），就不用濾鏡壓色 -->
<img class="logo {size}" class:own={hasAsset(artStyle.id, 'ui/logo.webp') && artStyle.id !== 'anime90'} src={asset('ui/logo.webp')} alt={t('虛擬闖天下')} draggable="false" />

<style>
  .logo {
    display: block;
    width: 100%;
    height: auto;
  }
</style>
