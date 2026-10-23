<script>
  // 角色資料卡：半身像、名字、介紹、能力值
  import { asset } from '../art-style.svelte.js';
  import { isUnlocked } from '../game/characters.js';
  import { t } from '../i18n.svelte.js';

  let { character, badge = '' } = $props();
  const open = $derived(isUnlocked(character));
  const STATS = [
    ['速度', 'speed'],
    ['力量', 'power'],
    ['體力', 'hp'],
  ];
</script>

<div class="panel detail">
  <div class="portrait">
    {#if open}
      {#key character.id}<img src={character.portraits.happy} alt="" />{/key}
    {:else}
      <img class="lock" src="{asset('ui/icons/lock.webp')}" alt="" />
    {/if}
  </div>
  <div class="info">
    <p class="name">{open ? character.name : '？？？'}</p>
    <p class="intro">{open ? character.intro : t('打倒第一大關的魔王就會出現。')}</p>
    {#if open}
      {#each STATS as [label, key] (key)}
        <div class="stat">
          {t(label)}
          <div class="bar">{#each { length: 5 } as _, i (i)}<i class:on={i < character.stats[key]}></i>{/each}</div>
        </div>
      {/each}
    {/if}
    {#if badge}<span class="badge">{badge}</span>{/if}
  </div>
</div>

<style>
  .detail {
    flex: 1;
    flex-direction: row;
    gap: 1.6cqh;
    padding: 1.6cqh;
    min-height: 0;
  }
  .portrait {
    aspect-ratio: 1;
    height: 100%;
    max-height: 30cqh;
    border: 0.45cqh solid var(--navy);
    outline: 0.3cqh solid var(--gold);
    outline-offset: -0.9cqh;
    background: linear-gradient(#5aa9ec, #9fd3ff);
    overflow: hidden;
    display: grid;
    place-items: center;
  }
  .portrait img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    animation: slide 0.18s ease-out;
  }
  .portrait img.lock {
    width: 50%;
    height: auto;
  }
  .info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.7cqh;
    min-width: 0;
  }
  .name {
    margin: 0;
    font-size: 5.6cqh;
    font-weight: 900;
    line-height: 1.1;
  }
  .intro {
    margin: 0 0 0.6cqh;
    font-size: 2.2cqh;
    font-weight: 800;
    color: var(--navy-soft);
  }
  .badge {
    align-self: flex-start;
    margin-top: auto;
    background: linear-gradient(#ffe56b, var(--gold) 55%, var(--gold-deep));
    border: 0.35cqh solid var(--navy);
    font-weight: 900;
    font-size: 2.6cqh;
    letter-spacing: 0.2em;
    padding: 0.4cqh 3cqh;
    clip-path: polygon(1cqh 0, 100% 0, calc(100% - 1cqh) 100%, 0 100%);
  }
  @keyframes slide {
    from {
      opacity: 0.4;
      transform: translateX(1.4cqh);
    }
  }
</style>
