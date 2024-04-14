<script>
  import "../app.pcss";
</script>

<div class="flex-col app dark:bg-neutral-800 justify-center min-h-screen">
  <main class="flex-col p-8">
    <slot />
  </main>
  <!-- <footer>
		<p>visit <a href="https://kit.svelte.dev">kit.svelte.dev</a> to learn SvelteKit</p>
	</footer> -->
</div>

<style>
  main {
    width: 100%;
    max-width: 64rem;
    margin: 0 auto;
    box-sizing: border-box;
  }
</style>
