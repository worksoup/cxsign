<script lang="ts">
  import { Input } from "$lib/components/ui/input/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import { Switch } from "$lib/components/ui/switch/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import { emit } from "@tauri-apps/api/event";
  import { createEventDispatcher } from "svelte";
  const dispatch = createEventDispatcher();
  let locationStr: string = "";
  let noRandomShift: boolean = false;
</script>

<div class="flex items-center space-x-2">
  <Switch id="no-random-shift" bind:checked={noRandomShift} />
  <Label for="no-random-shift">禁用位置偏移</Label>
  <Input bind:value={locationStr} inputmode="text" placeholder="位置" />
  <Button
    on:click={async () => {
      dispatch("sign");
      await emit("sign:location", {
        location_str: locationStr,
        no_random_shift: noRandomShift,
      });
    }}
  >
    签到
  </Button>
</div>
