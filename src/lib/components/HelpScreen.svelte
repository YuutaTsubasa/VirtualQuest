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
        <li><b>{t('連段')}</b>{t('：連按攻擊，打中後接第二、三下，第三下會把敵人踢倒。')}</li>
        <li><b>{t('跑步')}</b>{t('：連按兩下左右（搖桿往同一邊推兩下）就會跑；跑步中按攻擊是衝刺攻擊、按跳跳得更遠。')}</li>
        <li><b>{t('跳踢')}</b>{t('：跳起來在空中按攻擊。')}</li>
        <li><b>{t('抓摔')}</b>{t('：敵人被打得搖晃時走過去就會抓住，按攻擊膝擊，第三下（或按反方向＋攻擊）丟出去，能撞倒其他敵人。')}</li>
        <li><b>{t('必殺')}</b>{t('：打倒身邊所有敵人（攻擊＋跳一起按也可以），打中會扣一點自己的體力。')}</li>
        <li>{t('打破路上的箱子會掉出補血的食物或寶石。')}</li>
        <li>{t('一個人玩時方向鍵＋Z 攻擊、X 跳、C 必殺也可以；多人同樂時第 1 支手把是 1P、第 2 支是 2P，依此類推。')}</li>
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
    flex: 1.6;
  }
  .tips-panel {
    flex: 1;
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
    background: var(--navy);
    color: var(--on-dark);
    padding: 1cqh 2.4cqh 1cqh 1.6cqh;
    white-space: nowrap;
    clip-path: polygon(0 0, 100% 0, calc(100% - 1cqh) 100%, 0 100%);
  }
  tbody td {
    background: var(--paper-white);
    border-top: 0.3cqh solid var(--navy);
    border-bottom: 0.3cqh solid var(--navy);
    text-align: center;
    padding: 1cqh 0.6cqh;
  }
  ul {
    margin: 0;
    padding-left: 2.6cqh;
    font-size: 2.3cqh;
    font-weight: 700;
    line-height: 1.55;
    display: flex;
    flex-direction: column;
    gap: 0.8cqh;
  }
</style>
