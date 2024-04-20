<script lang="ts">
  import { Input } from "$lib/components/ui/input/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import { emit } from "@tauri-apps/api/event";
  import { createEventDispatcher } from "svelte";
  const dispatch = createEventDispatcher();
  let locationStr: string = "";
</script>

<div class="flex items-center space-x-2">
  <Input bind:value={locationStr} inputmode="text" placeholder="位置" />
  <Button
    on:click={async () => {
      dispatch("sign");
      await emit("sign:location", {
        location_str: locationStr,
      });
    }}
  >
    签到
  </Button>
</div>
