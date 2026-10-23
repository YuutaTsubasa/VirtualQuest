<script>
  // 「虛擬闖天下」標誌（art-work/ui-concept-v1/logo-transparent.png 裁掉透明邊，ui/logo.webp）
  import { asset } from '../art-style.svelte.js';
  import { t } from '../i18n.svelte.js';
  let { size = 'small' } = $props();
</script>

<img class="logo {size}" src={asset('ui/logo.webp')} alt={t('虛擬闖天下')} draggable="false" />

<style>
  .logo {
    display: block;
    width: 100%;
    height: auto;
  }
</style>
