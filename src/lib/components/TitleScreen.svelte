<script>
  // 標題畫面：標誌、主選單，背景是隨機一大關，左右各一位角色（每次隨機）
  import { asset } from '../art-style.svelte.js';
  import { CHARACTERS } from '../game/characters.js';
  import Logo from './Logo.svelte';
  import Menu from './Menu.svelte';
  import SpritePose from './SpritePose.svelte';
  import WorldScene from './WorldScene.svelte';
  import ControlHint from './ControlHint.svelte';

  let { items } = $props();

  const pool = CHARACTERS.filter((c) => !c.hidden);
  const leftIndex = Math.floor(Math.random() * pool.length);
  // 右邊挑一位不一樣的
  const left = pool[leftIndex];
  const right = pool[(leftIndex + 1 + Math.floor(Math.random() * (pool.length - 1))) % pool.length];

  const world = Math.floor(Math.random() * 6);
  // 有標題主視覺（ui/title-scene）就用插圖，沒有就用關卡背景＋左右兩位角色的動作圖
  const scene = $derived(asset('ui/title-scene.webp'));
</script>

<section class="screen title">
  {#if scene}
    <img class="menu-bg" src={scene} alt="" />
  {:else}
    <WorldScene {world} />
    <div class="hero left"><SpritePose character={left} pose="spike" size="64cqh" /></div>
    <div class="hero right"><SpritePose character={right} pose="idle" size="58cqh" flip /></div>
  {/if}

  <div class="center">
    <Logo size="big" />
    <Menu {items} width="36cqw" />
  </div>

  <div class="foot"><ControlHint /></div>
</section>

<style>
  .title {
    justify-content: flex-start;
  }
  .center {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2cqh;
    margin-top: 3cqh;
  }
  .hero {
    position: absolute;
    bottom: 4cqh;
    filter: drop-shadow(0 1cqh 0.6cqh rgba(80, 50, 20, 0.35));
  }
  .hero.left {
    left: -2cqw;
    animation: float 2.4s ease-in-out infinite alternate;
  }
  .hero.right {
    right: 1cqw;
  }
  .foot {
    position: absolute;
    bottom: 2.6cqh;
    left: 50%;
    translate: -50% 0;
  }
  @keyframes float {
    to {
      transform: translateY(-2cqh);
    }
  }
</style>
