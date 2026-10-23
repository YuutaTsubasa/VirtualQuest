<script>
  // 齒輪：打開設定視窗（選單畫面的角落都有）
  import { asset } from '../art-style.svelte.js';
  import { t } from '../i18n.svelte.js';
  import { openSettings } from '../ui.svelte.js';
  import { play } from '../game/audio.js';
</script>

<button
  type="button"
  class="settings-btn"
  aria-label={t('設定')}
  title={t('設定')}
  onclick={() => {
    play('select');
    openSettings();
  }}
>
  <img src={asset('ui/icons/settings.webp')} alt="" />
</button>

<style>
  .settings-btn {
    width: 9cqh;
    height: 9cqh;
    flex: none;
    padding: 0.9cqh;
    border: 0.45cqh solid var(--navy);
    border-radius: 50%;
    background: var(--cream);
    box-shadow: 0 0.5cqh 0 rgba(12, 29, 75, 0.35);
    transition: transform 0.15s;
  }
  .settings-btn:hover {
    transform: rotate(30deg);
  }
  .settings-btn img {
    width: 100%;
    height: 100%;
    display: block;
  }
</style>
