<script lang="ts">
  import { cn } from "$lib/utils";
  import type { HTMLAttributes } from "svelte/elements";

  type $$Props = HTMLAttributes<HTMLDivElement>;
  export let clazz = "";
  export { clazz as class };
</script>

<div class={cn("grid-rows-3 grid-col-3", clazz)}>

</div>
