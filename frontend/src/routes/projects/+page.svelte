
<script>
  import { onMount } from 'svelte';
  
  async function fetchProjects() {
    await fetch('http://localhost:3000/projects/retrieve_projects')
    .then((res) => res.json())
    .then(data => {
      console.log(data);
      return data;
    });
  }

  async function createProject() {
    await fetch('http://localhost:3000/projects/create_project')
    .then((res) => res.json())
  }

  let data = $state();
  onMount(async () => {
    data = fetchProjects();
  });
</script>

<button onclick={()=>{console.log("AHHHHH"); }}>
  Add New Project
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
