<script>
  let {children} = $props();
</script>


<header class="top-panel">
  <h1>Cadmus Client</h1>
</header>
<div class="outer">
  <nav class="side-panel">
    <a href="/projects">Projects</a>
    <a href="/account">Account</a>
    <a href="/settings">Settings</a>
  </nav>

  <div class="content">
    {@render children()}
  </div>
</div>

<style>
  .outer {
    display: flex;
  }

  .top-panel {
  }

  .side-panel {
    flex-direction: column;
    display: flex;
  }

  .content {
    display: flex;
    flex-direction: column;
    width: 100%;
  }
</style>

