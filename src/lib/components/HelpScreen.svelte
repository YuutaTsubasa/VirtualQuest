<script>
  // 操作說明：鍵盤／手把／觸控對照表與小技巧
  import { asset } from '../art-style.svelte.js';
  import ScreenFrame from './ScreenFrame.svelte';
  import { t } from '../i18n.svelte.js';
  import BackButton from './BackButton.svelte';
  import ControlHint from './ControlHint.svelte';

  let { onback } = $props();
  const ROWS = [
    ['移動', 'W A S D', '方向鍵', '十字鍵／左搖桿', '左下搖桿'],
    ['攻擊', 'J', '數字鍵 1／,', 'X／RB', '「攻擊」'],
    ['跳', 'K', '數字鍵 2／.', 'A／LB', '「跳」'],
    ['必殺', 'L', '數字鍵 3／/', 'B／Y／RT', '「必殺」'],
    ['暫停', 'Esc／P', 'Esc／P', 'Start', '上方暫停鈕'],
  ];
</script>

<ScreenFrame title={t('操作說明')} subtitle={t('鍵盤・手把・觸控都能玩')}>
  <div class="panel keys-panel">
    <div class="panel-head">{t('操作方式')}</div>
    <div class="panel-body">
      <table>
        <thead>
          <tr>
            <th></th>
            <th><img src="{asset('ui/icons/keyboard.webp')}" alt="" />{t('鍵盤 1P')}</th>
            <th><img src="{asset('ui/icons/keyboard.webp')}" alt="" />{t('鍵盤 2P')}</th>
            <th><img src="{asset('ui/icons/gamepad.webp')}" alt="" />{t('手把')}</th>
            <th><img src="{asset('ui/icons/touch.webp')}" alt="" />{t('觸控')}</th>
          </tr>
        </thead>
        <tbody>
          {#each ROWS as [label, ...cells] (label)}
            <tr>
              <th>{t(label)}</th>
              {#each cells as c, i (i)}<td>{t(c)}</td>{/each}
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>

  <div class="panel tips-panel">
    <div class="panel-head">{t('小技巧')}</div>
    <div class="panel-body">
      <ul>
        <li><b>{t('連段')}</b>{t('：連按攻擊打三下，第三下把敵人踢倒。單挑時傷害最高。')}</li>
        <li><b>{t('跑步')}</b>{t('：連按兩下左右就會跑。跑步中按攻擊是衝刺攻擊，能穿過一整排（打空收招很久）。')}</li>
        <li><b>{t('跳踢')}</b>{t('：空中按攻擊。用來接近或逃出包圍，傷害較低，魔王會對空。')}</li>
        <li><b>{t('抓摔')}</b>{t('：敵人搖晃時走過去抓住，按攻擊膝擊，第三下丟出去撞倒別人。')}</li>
        <li><b>{t('必殺')}</b>{t('：每人不同。普通攻擊打中、被打都會集必殺氣，集滿才能放。')}</li>
        <li><b>{t('武器')}</b>{t('：站在旁邊按攻擊撿起來，打幾下就會壞；跑步中按攻擊丟出去，被打倒會掉。')}</li>
        <li><b>{t('大隻佬')}</b>{t('：拳頭打不停他，用踢、跳踢、衝刺或丟人。')}</li>
        <li>{t('打破路上的箱子會掉出補血的食物或寶石。')}</li>
        <li>{t('一個人玩也能用方向鍵＋Z／X／C；手把依序是 1P、2P…')}</li>
        <li>{t('打倒第一大關的魔王，會有隱藏角色加入。')}</li>
      </ul>
    </div>
  </div>

  {#snippet foot()}
    <BackButton onclick={onback} />
    <span></span>
    <ControlHint />
  {/snippet}
</ScreenFrame>

<style>
  .keys-panel {
    flex: 1.35;
  }
  .tips-panel {
    flex: 1.15;
  }
  /* 字多的語言（英文）塞不下時可以捲動，不要超出框 */
  .tips-panel .panel-body {
    overflow-y: auto;
    min-height: 0;
  }
  table {
    width: 100%;
    border-collapse: separate;
    border-spacing: 0 0.8cqh;
    font-size: 2.4cqh;
    font-weight: 800;
  }
  thead th {
    font-size: 2.3cqh;
    white-space: nowrap;
  }
  thead img {
    display: block;
    width: 6cqh;
    margin: 0 auto 0.4cqh;
  }
  tbody th {
    background: var(--ink-fill);
    color: var(--on-dark);
    padding: 1cqh 2.4cqh 1cqh 1.6cqh;
    white-space: nowrap;
    clip-path: polygon(0 0, 100% 0, calc(100% - 1cqh) 100%, 0 100%);
  }
  tbody td {
    background: var(--paper-white);
    border-top: 0.3cqh solid var(--edge);
    border-bottom: 0.3cqh solid var(--edge);
    text-align: center;
    padding: 1cqh 0.6cqh;
  }
  ul {
    margin: 0;
    padding-left: 2.6cqh;
    font-size: 2.5cqh;
    font-weight: 700;
    line-height: 1.5;
    display: flex;
    flex-direction: column;
    gap: 0.9cqh;
  }
  :global(html[lang='ja']) ul {
    font-size: 2.3cqh;
    line-height: 1.45;
    gap: 0.6cqh;
  }
  /* 英文字比較長：字小一點才塞得進同一個框 */
  :global(html[lang='en']) ul {
    font-size: 2cqh;
    line-height: 1.35;
    gap: 0.5cqh;
  }
</style>
