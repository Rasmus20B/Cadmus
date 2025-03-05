
<script>
  async function fetchProjects() {
    return await fetch('http://localhost:3000/projects').then((res) => res.json());
  }
</script>

<button on:click={()=>{fetchProjects()}}>
  Get Projects
</button>

<div>
{#await fetchProjects()}
  <p>loading projects...</p>
{:then data}
<div style='background="#ffaaff"'>
  <ul>
    {#each data as item}
      {console.log(data)}
      <li>Project: {item.name}</li>
    {/each}
  </ul>
</div>
{:catch}
  <p>Could not load projects :(</p>
{/await}
</div>
