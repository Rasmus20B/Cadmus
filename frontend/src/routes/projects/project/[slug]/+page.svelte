
<script>
  import { page } from '$app/stores';
  let project_name = "";
  page.subscribe((value) => {
    project_name = value.params.slug;
  });
</script>

<main>
  <div class="container">
  <header class="project-header">
    <h1>{project_name}</h1>
  </header>

  <div class="split-view">
    <aside class="file-list">
      <ul>
        <li>Test 1</li>
        <li>Test 2</li>
        <li>Test 3</li>
      </ul>
    </aside>

    <textarea class="file-content">
    </textarea>
</main>

<style>
  .container {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }

  .project-header {
    background: #222;
    color: white;
    padding: 10px;
    text-align: left;
  }

  .split-view {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .file-list {
    width: 250px;
    background: #f4f4f4;
    padding: 10px;
    border-right: 1px solid #ccc;
    overflow-y: auto;
  }

  .file-content {
    flex-grow: 1;
    padding: 20px;
    background: white;
    overflow-y: auto;
  }
</style>
