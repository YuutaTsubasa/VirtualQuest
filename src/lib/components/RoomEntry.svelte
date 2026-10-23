<script>
  // 開設房間／加入房間：左邊玩家資料，右邊選角色
  import { onMount } from 'svelte';
  import { net, inTauri, connect, disconnect, startHost, stopHost, hostStatus, toast, load, store, DEFAULT_PORT, checkOnline, hostOnline, joinOnline } from '../net.svelte.js';
  import { SIGNAL_SERVER, WEB_URL, roomCodeOf } from '../site.js';
  import { CHARACTERS, isUnlocked, getCharacter } from '../game/characters.js';
  import { play } from '../game/audio.js';
  import { t, serverText } from '../i18n.svelte.js';
  import ScreenFrame from './ScreenFrame.svelte';
  import BackButton from './BackButton.svelte';
  import CharacterGrid from './CharacterGrid.svelte';

  // page = host | join；code：從分享連結（#join/房號）打開時帶入的房號
  let { page, code = '', onback } = $props();
  const isHost = $derived(page === 'host');

  // 從房主的伺服器打開的網頁：位址直接帶入（GitHub Pages 上的網頁版不是房主的伺服器）
  const servedByServer =
    !inTauri && location.protocol.startsWith('http') && !import.meta.env.DEV && !(WEB_URL && location.href.startsWith(WEB_URL));

  const available = CHARACTERS.filter(isUnlocked);
  let name = $state(load('vq_name', ''));
  let port = $state(load('vq_port', String(DEFAULT_PORT)));
  // svelte-ignore state_referenced_locally
  let address = $state(code || (servedByServer ? location.host : load('vq_address', '')));
  // 開房方式：線上（房號，WebRTC）或區網（桌面 App 內建伺服器／自架伺服器）
  let hostMode = $state(load('vq_host_mode', 'online'));
  // 線上房間只有配對服務在的時候才能開（沒設定配對服務就整個藏起來）
  const online = $derived(isHost && !!net.online && hostMode === 'online');
  let hostKey = $state('');
  let character = $state(
    available.find((c) => c.id === load('vq_character'))?.id ?? available[Math.floor(Math.random() * available.length)].id,
  );
  let cursor = $state(CHARACTERS.findIndex((c) => c.id === character));
  let busy = $state(false);
  const connecting = $derived(!net.room && (net.status === 'connecting' || net.status === 'open'));
  const chosen = $derived(getCharacter(character));

  onMount(() => {
    hostStatus().catch(() => {});
    checkOnline();
  });

  function pickMode(m) {
    play('select');
    hostMode = m;
    store('vq_host_mode', m);
  }

  const playerName = () => name.trim() || t('玩家');

  function remember() {
    store('vq_name', name);
    store('vq_character', character);
  }

  async function submit() {
    if (busy || connecting) return;
    play('select');
    remember();
    if (online) return hostOnline({ name: playerName(), character });
    if (!isHost && roomCodeOf(address)) return joinRoomCode(false);
    if (isHost && inTauri) {
      if (net.hostInfo) {
        connect(`localhost:${net.hostInfo.port}`, { name: playerName(), hostKey: net.hostInfo.host_key, character });
        return;
      }
      store('vq_port', port);
      busy = true;
      try {
        await startHost(port, playerName(), character);
      } catch (error) {
        toast(serverText(String(error)));
      } finally {
        busy = false;
      }
      return;
    }
    if (!address.trim()) return toast(t('請輸入房主的位址，例如 {example}', { example: `192.168.1.5:${DEFAULT_PORT}` }));
    if (isHost && !hostKey.trim()) return toast(t('請輸入伺服器顯示的房主金鑰'));
    if (!servedByServer) store('vq_address', address);
    connect(address, { name: playerName(), hostKey: isHost ? hostKey.trim() : undefined, character });
  }

  // 加入線上房間（輸入房號或分享連結）
  function joinRoomCode(spectate) {
    if (net.online === false) return toast(t('線上房間暫時無法使用，請稍後再試'));
    store('vq_address', address);
    joinOnline(roomCodeOf(address), { name: playerName(), spectate, character });
  }

  function spectate() {
    if (!address.trim()) return toast(net.online ? t('請輸入房號或房主的位址') : t('請輸入房主的位址'));
    remember();
    if (roomCodeOf(address)) return joinRoomCode(true);
    connect(address, { name: playerName(), spectate: true });
  }

  async function closeRoom() {
    busy = true;
    try {
      await stopHost();
    } catch (error) {
      toast(serverText(String(error)));
    } finally {
      busy = false;
    }
  }

  function back() {
    if (connecting) disconnect();
    onback();
  }

  const hint = $derived(
    connecting
      ? net.code
        ? t('連線到房間 {code} 中…', { code: net.code })
        : t('連線到 {address} 中…', { address: net.address })
      : online
        ? t('開房後會拿到 5 碼房號與分享連結，朋友在任何地方都能加入。')
        : isHost
          ? inTauri
            ? t('開房後，可在大廳分享網址邀請朋友。')
            : t('在網頁開房：輸入伺服器位址與啟動時印出的房主金鑰。')
          : net.online
            ? t('輸入房號、分享連結或區網位址；沒有座位時會改成觀戰。')
            : t('輸入房主分享的網址；沒有座位時會改成觀戰。'),
  );
</script>

<ScreenFrame title={isHost ? t('開設房間') : t('加入房間')} subtitle={t('設定玩家資料與角色')}>
  <form
    class="panel profile"
    onsubmit={(e) => {
      e.preventDefault();
      submit();
    }}
  >
    <div class="panel-head">{t('玩家資料')}</div>
    <div class="panel-body">
      <label class="field">{t('玩家名稱')}
        <input bind:value={name} placeholder={t('玩家')} maxlength="16" autocomplete="nickname" />
      </label>

      {#if isHost && SIGNAL_SERVER}
        <div class="field">{t('開房方式')}
          <div class="modes" role="radiogroup">
            <button type="button" role="radio" aria-checked={online} class="vbtn small" class:active={online} disabled={!net.online} onclick={() => pickMode('online')}>{t('線上（房號）')}</button>
            <button type="button" role="radio" aria-checked={!online} class="vbtn small" class:active={!online} onclick={() => pickMode('lan')}>{inTauri ? t('區網（桌面 App）') : t('區網（自架伺服器）')}</button>
          </div>
          {#if net.online === null}<small>{t('檢查線上房間中…')}</small>{:else if !net.online}<small>{t('線上房間暫時無法使用，請稍後再試。')}</small>{/if}
        </div>
      {/if}

      {#if online}
        <p class="note">{t('比賽在你的瀏覽器裡跑，朋友用 WebRTC 直接連到你。')}<small>{t('開著這個畫面房間才在，關掉分頁房間就關了。')}</small></p>
      {:else if isHost && inTauri}
        {#if net.hostInfo}
          <p class="note">{t('房間已在通訊埠 {port} 開著。', { port: net.hostInfo.port })}</p>
          <button type="button" class="vbtn small" onclick={closeRoom} disabled={busy}>{t('關閉房間')}</button>
        {:else}
          <label class="field">{t('通訊埠')}
            <input bind:value={port} type="number" min="1" max="65535" required />
            <small>{t('預設 {port}', { port: DEFAULT_PORT })}</small>
          </label>
        {/if}
      {:else}
        <label class="field">{isHost ? t('伺服器位址') : net.online ? t('房號或網址') : t('房間網址')}
          <input
            bind:value={address}
            placeholder={!isHost && net.online ? t('{code} 或 {address}', { code: 'K7QX2', address: `192.168.1.5:${DEFAULT_PORT}` }) : `192.168.1.5:${DEFAULT_PORT}`}
            readonly={servedByServer && !code}
            spellcheck="false"
          />
        </label>
        {#if isHost}
          <label class="field">{t('房主金鑰')}
            <input bind:value={hostKey} placeholder={t('伺服器啟動時印出的金鑰')} />
          </label>
        {/if}
      {/if}

      <div class="chosen">
        <div class="frame-pic"><img src={chosen.portraits.normal} alt="" /></div>
        <div>
          <b>{chosen.name}</b>
          <small>{chosen.intro}</small>
          <span class="badge">{t('已選擇')}</span>
        </div>
      </div>
    </div>
  </form>

  <div class="panel roster">
    <div class="panel-head">{t('選擇角色')}</div>
    <div class="panel-body">
      <CharacterGrid bind:cursor selected={character} showLocked={false} onpick={(c) => (character = c.id)} />
      <p class="tip">{t('點選頭像，選擇你的角色')}</p>
    </div>
  </div>

  {#snippet foot()}
    <BackButton onclick={back} />
    <div class="hint-bar">{hint}</div>
    <div class="actions">
      {#if connecting}
        <button type="button" class="vbtn small" onclick={disconnect}>{t('取消')}</button>
      {:else if !isHost}
        <button type="button" class="vbtn small" onclick={spectate}>{t('觀戰')}</button>
      {/if}
      <button type="button" class="vbtn primary" disabled={busy || connecting} onclick={submit}>
        {isHost ? (!online && net.hostInfo ? t('回到房間') : t('開設房間')) : t('加入房間')}
      </button>
    </div>
  {/snippet}
</ScreenFrame>

<style>
  .profile {
    flex: 1;
  }
  /* 區網開房的欄位比較多，放不下時面板裡面捲動，不要超出框 */
  .profile .panel-body {
    overflow-y: auto;
    gap: 0.8cqh;
  }
  .roster {
    flex: 1.75;
  }
  .note {
    margin: 0;
    font-weight: 800;
    font-size: 2.3cqh;
  }
  .note small {
    display: block;
    margin-top: 0.4cqh;
    font-weight: 700;
    font-size: 1.9cqh;
    color: var(--muted);
  }
  .modes {
    display: flex;
    gap: 1cqh;
  }
  .modes .vbtn {
    flex: 1;
    min-width: 0;
    padding-inline: 1.6cqh;
    font-size: 2.4cqh;
    white-space: nowrap;
  }
  .chosen {
    margin-top: auto;
    display: flex;
    gap: 1.6cqh;
    align-items: center;
    border-top: 0.35cqh solid var(--navy);
    padding-top: 1.4cqh;
  }
  .frame-pic {
    width: 13cqh;
    height: 13cqh;
    flex: none;
    border: 0.45cqh solid var(--navy);
    outline: 0.3cqh solid var(--gold);
    outline-offset: -0.9cqh;
    background: linear-gradient(#5aa9ec, #9fd3ff);
    overflow: hidden;
  }
  .frame-pic img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .chosen > div:last-child {
    display: flex;
    flex-direction: column;
    gap: 0.5cqh;
  }
  .chosen b {
    font-size: 4.2cqh;
    line-height: 1.1;
  }
  .chosen small {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    font-size: 2.1cqh;
    font-weight: 800;
    color: var(--navy-soft);
  }
  .badge {
    align-self: flex-start;
    background: linear-gradient(#ffe56b, var(--gold) 55%, var(--gold-deep));
    border: 0.35cqh solid var(--navy);
    font-weight: 900;
    font-size: 2.4cqh;
    letter-spacing: 0.2em;
    padding: 0.3cqh 2.6cqh;
    clip-path: polygon(1cqh 0, 100% 0, calc(100% - 1cqh) 100%, 0 100%);
  }
  .tip {
    margin: 0;
    text-align: center;
    font-weight: 800;
    font-size: 2.1cqh;
    letter-spacing: 0.2em;
  }
  .actions {
    display: flex;
    gap: 1.4cqh;
    align-items: center;
  }
</style>
