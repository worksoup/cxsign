<script lang="ts">
  import { Button } from "$lib/components/ui/button/index.js";
  import { createEventDispatcher } from "svelte";
  const dispatch = createEventDispatcher();
</script>

<Button
  on:click={(e) => {
    dispatch("sign", e.detail);
  }}
>
  签到
</Button>
