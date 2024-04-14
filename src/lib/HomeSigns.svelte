<script lang="ts">
  import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
  import * as Table from "$lib/components/ui/table/index.js";
  import { cancel } from "@tauri-apps/plugin-barcode-scanner";
  import { Button } from "$lib/components/ui/button/index.js";
  import type { RawSign, RawSignPair } from "./commands/sign";
  import SignLayout from "./sign/SignLayout.svelte";
  import type { AccountPair } from "./commands/account";
  import { Page } from "./commands/tools";
  import { createEventDispatcher } from "svelte";
  import { Skeleton } from "$lib/components/ui/skeleton/index.js";
  export let signs: RawSignPair[] = [];
  export let state: Page = Page.home;
  export let scanning = false;
  export let updating = false;
  let sign: RawSign;
  let accounts: AccountPair[] = [];
  const dispatch = createEventDispatcher();
  async function onRowClick(s: RawSignPair) {
    window.history.pushState(
      { state: Page.courseSigns },
      "",
      "?state=1&page=SignLayout"
    );
    sign = s.sign;
    accounts = s.unames;
    state = Page.sign;
  }
</script>

{#if state == Page.home}
  <div class="items-center justify-center">
    <ScrollArea class="h-[77.5vh] rounded-md border mb-2">
      <Table.Root>
        <Table.Body>
          {#if updating}
            {#each [0, 1, 2, 3, 4, 5, 6, 7, 8, 9] as { }}
              <Table.Row>
                <Table.Cell>
                  <div class="flex items-center space-x-4">
                    <Skeleton class="h-12 w-12 rounded-full" />
                    <div class="space-y-2">
                      <Skeleton class="h-4 w-[250px]" />
                      <Skeleton class="h-4 w-[200px]" />
                    </div>
                  </div>
                </Table.Cell>
              </Table.Row>
            {/each}
          {:else}
            {#each signs as sign (sign.sign.name)}
              <Table.Row>
                <Table.Cell
                  on:click={() => {
                    onRowClick(sign);
                  }}
                >
                  <p class="truncate max-w-[80vw]">
                    {sign.sign.name}
                  </p>
                </Table.Cell>
              </Table.Row>
            {/each}
          {/if}
        </Table.Body>
      </Table.Root>
    </ScrollArea>
  </div>
  <div class="flex flex-row-reverse justify-center">
    <div class="flex grow space-x-2">
      <!-- <div class="grow w-2/3"></div> -->
      <Button
        class="grow"
        disabled={updating}
        on:click={() => {
          dispatch("updateSigns");
        }}
      >
        刷新
      </Button>
    </div>
  </div>
{:else if sign}
  <SignLayout bind:scanning {sign} {accounts} bind:state></SignLayout>
{/if}
