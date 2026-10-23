<script>
  // 選單畫面的共用版型：左上標誌、右上標題橫幅、金色分隔線、中間內容、底部（返回／提示／主要按鈕）
  import { asset } from '../art-style.svelte.js';
  import Logo from './Logo.svelte';
  import SettingsButton from './SettingsButton.svelte';

  let { title, subtitle = '', children, foot } = $props();
  // 標題太長（英文、日文）時縮小字級，免得超出橫幅：中文字約 1.2 字寬（含字距），英數約 0.6
  const titleSize = $derived.by(() => {
    let w = 0;
    for (const ch of title) w += /[　-鿿＀-￯]/.test(ch) ? 1.2 : 0.62;
    return `min(7.4cqh, ${(44 / Math.max(w, 1)).toFixed(2)}cqw)`;
  });
</script>

<section class="screen framed">
  <img class="menu-bg" src="{asset('game/court.webp')}" alt="" />
  <div class="frame">
    <header class="frame-head">
      <Logo />
      <div class="banner">
        <h2 style:font-size={titleSize}>{title}</h2>
        {#if subtitle}<p>{subtitle}</p>{/if}
      </div>
      <SettingsButton />
    </header>
    <div class="frame-main">{@render children()}</div>
    {#if foot}<footer class="frame-foot">{@render foot()}</footer>{/if}
  </div>
</section>

<style>
  .framed {
    justify-content: stretch;
  }
  .frame-head {
    display: grid;
    grid-template-columns: 34cqw 1fr auto;
    align-items: center;
    gap: 3cqw;
    position: relative;
    padding-bottom: 1.4cqh;
  }
  /* 左上角的 Logo：跟標題橫幅一樣高，不壓到金色分隔線 */
  .frame-head :global(.logo) {
    width: auto;
    max-width: 100%;
    height: 17cqh;
    object-fit: contain;
    object-position: left center;
  }
  /* 金色分隔線 */
  .frame-head::after {
    content: '';
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    height: 0.8cqh;
    background: linear-gradient(var(--gold), var(--gold-deep));
    border-top: 0.25cqh solid var(--navy);
    border-bottom: 0.25cqh solid var(--navy);
  }

  /* 標題橫幅：米白外框＋深藍六角形，兩端有斜線 */
  .banner {
    --cut: 4.5cqh;
    position: relative;
    height: 17cqh;
    background: var(--cream);
    clip-path: polygon(var(--cut) 0, calc(100% - var(--cut)) 0, 100% 50%, calc(100% - var(--cut)) 100%, var(--cut) 100%, 0 50%);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    isolation: isolate;
    filter: drop-shadow(0 0.5cqh 0 rgba(12, 29, 75, 0.4));
  }
  .banner::before {
    content: '';
    position: absolute;
    inset: 0.9cqh 1.2cqh;
    z-index: -1;
    background:
      repeating-linear-gradient(115deg, transparent 0 0.7cqh, rgba(223, 230, 245, 0.85) 0.7cqh 1.3cqh) left 5cqh center / 4.6cqh 56% no-repeat,
      repeating-linear-gradient(115deg, transparent 0 0.7cqh, rgba(223, 230, 245, 0.85) 0.7cqh 1.3cqh) right 5cqh center / 4.6cqh 56% no-repeat,
      var(--navy);
    clip-path: polygon(calc(var(--cut) - 1cqh) 0, calc(100% - var(--cut) + 1cqh) 0, 100% 50%, calc(100% - var(--cut) + 1cqh) 100%, calc(var(--cut) - 1cqh) 100%, 0 50%);
  }
  h2 {
    margin: 0;
    color: var(--on-dark);
    font-size: 7.4cqh;
    font-weight: 900;
    letter-spacing: 0.18em;
    text-indent: 0.18em;
    line-height: 1.1;
    text-shadow: 0 0.5cqh 0 var(--navy-deep), 0 0 1.4cqh rgba(255, 255, 255, 0.15);
  }
  p {
    margin: 0.3cqh 0 0;
    padding: 0.2cqh 4cqh;
    background: var(--navy-deep);
    color: var(--on-dark);
    font-size: 2.3cqh;
    font-weight: 800;
    letter-spacing: 0.35em;
    text-indent: 0.35em;
    border-top: 0.25cqh solid var(--cream);
    border-bottom: 0.25cqh solid var(--cream);
  }
</style>
