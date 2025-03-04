
<script>
  async function fetchProjects() {
    return fetch('http://localhost:3000/projects').then(res => res.text());
  }

  const data = fetchProjects();
</script>

<button on:click={()=>{fetchProjects()}}>
  Get Projects
</button>

{#await data}
  <p>Fetching projects...</p>
{:then data}
  <p>{data}</p>
{/await}
