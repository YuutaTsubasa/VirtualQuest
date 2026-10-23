<script>
  // 「虛擬闖天下」標誌。Logo 圖還沒畫好之前先用文字排（ui/logo.webp 交件後換成圖）
  import { asset } from '../art-style.svelte.js';
  import { hasAsset } from '../style-assets.js';
  import { artStyle } from '../art-style.svelte.js';
  import { t } from '../i18n.svelte.js';
  let { size = 'small' } = $props();
  const image = $derived(hasAsset(artStyle.id, 'ui/logo.webp') ? asset('ui/logo.webp') : null);
</script>

{#if image}
  <img class="logo {size}" src={image} alt={t('虛擬闖天下')} draggable="false" />
{:else}
  <div class="logo text {size}" role="img" aria-label={t('虛擬闖天下')}>
    <span class="zh">虛擬闖天下</span>
    <span class="en">VIRTUAL QUEST</span>
  </div>
{/if}

<style>
  .logo {
    display: block;
    width: 100%;
    height: auto;
    filter: drop-shadow(0 0.6cqh 0.4cqh rgba(12, 29, 75, 0.35));
  }
  .logo.big {
    width: 50cqw;
  }
  .text {
    display: flex;
    flex-direction: column;
    align-items: center;
    line-height: 1;
    font-weight: 900;
    user-select: none;
  }
  .zh {
    font-size: 6.4cqh;
    letter-spacing: 0.08em;
    color: var(--gold);
    -webkit-text-stroke: 0.5cqh var(--navy);
    paint-order: stroke fill;
    font-style: italic;
  }
  .en {
    margin-top: 0.4cqh;
    font-size: 2cqh;
    letter-spacing: 0.4em;
    color: var(--cream, #fff);
    -webkit-text-stroke: 0.3cqh var(--navy);
    paint-order: stroke fill;
  }
  .big .zh {
    font-size: 15cqh;
  }
  .big .en {
    font-size: 4cqh;
  }
</style>
