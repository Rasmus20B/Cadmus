
<script>
  import { onMount } from 'svelte';
  
  async function fetchProjects() {
    return await fetch('http://localhost:3000/projects/retrieve_projects')
    .then((res) => res.json())
    .then(data => {
      console.log(data);
      return data;
    });
  }

  let data = $state();
  onMount(async () => {
    data = fetchProjects();
  });
</script>

<button onclick={()=>{console.log("AHHHHH")}}>
  Get Projects
</button>

<div>
{#await data}
  <p>loading projects...</p>
{:then data}
<div class="projects">
  <ul>
    {#each data as item}
      <div class="project">
        <a href="/projects/project/{item.name}">Project #{item.id}: {item.name}</a>
      </div>
    {/each}
  </ul>
</div>
{:catch}
  <p>Could not load projects :(</p>
{/await}
</div>

<style>
  .projects {
    display: grid;
    gap: 1rem;
    margin-block-start: 1rem;
  }

  .project {
    position: relative;
  }

</style>
