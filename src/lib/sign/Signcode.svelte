<script lang="ts">
  import { Input } from "$lib/components/ui/input/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import { emit } from "@tauri-apps/api/event";
  import { isStringEmpty } from "$lib/commands/account";
  import { createEventDispatcher } from "svelte";
  const dispatch = createEventDispatcher();
  let signcode: string;
</script>

<div class="flex items-center space-x-2">
  <form
    on:submit|preventDefault={async () => {
      dispatch("sign");
      await emit("sign:signcode", signcode);
    }}
  >
    <Input bind:value={signcode} inputmode="numeric" placeholder="签到码" />
    <Button type="submit" disabled={isStringEmpty(signcode)}>签到</Button>
  </form>
</div>
