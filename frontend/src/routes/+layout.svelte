<script>
  let {children} = $props();
</script>

<nav>
  <a href="/">home</a>
  <a href="/projects">Projects</a>
</nav>

{@render children()}
