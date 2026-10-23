<script>
  // 疊在比賽畫面上的視窗（暫停、結果、連線選單）：深藍標題條＋米白面板
  let { title, subtitle = '', wide = false, children } = $props();
</script>

<section class="screen overlay">
  <div class="dialog panel" class:wide>
    <div class="dialog-title">
      <h2>{title}</h2>
      {#if subtitle}<p>{subtitle}</p>{/if}
    </div>
    <div class="dialog-body">{@render children()}</div>
  </div>
</section>

<style>
  .dialog {
    min-width: 52cqw;
    align-items: center;
    padding-bottom: 3cqh;
  }
  .dialog.wide {
    min-width: 76cqw;
  }
  .dialog-title {
    align-self: stretch;
    margin: 1.2cqh 1.2cqh 0;
    padding: 1.4cqh 6cqh 1.2cqh;
    background:
      repeating-linear-gradient(115deg, transparent 0 0.6cqh, rgba(223, 230, 245, 0.85) 0.6cqh 1.1cqh) left 3cqh center / 4cqh 50% no-repeat,
      repeating-linear-gradient(115deg, transparent 0 0.6cqh, rgba(223, 230, 245, 0.85) 0.6cqh 1.1cqh) right 3cqh center / 4cqh 50% no-repeat,
      var(--navy);
    text-align: center;
    clip-path: polygon(2cqh 0, calc(100% - 2cqh) 0, 100% 50%, calc(100% - 2cqh) 100%, 2cqh 100%, 0 50%);
  }
  h2 {
    margin: 0;
    color: var(--on-dark);
    font-size: 6cqh;
    font-weight: 900;
    letter-spacing: 0.2em;
    text-indent: 0.2em;
  }
  p {
    margin: 0.4cqh 0 0;
    color: var(--gold);
    font-weight: 800;
    font-size: 2.3cqh;
    letter-spacing: 0.2em;
  }
  .dialog-body {
    padding: 3cqh 4cqh 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2.4cqh;
  }
</style>
