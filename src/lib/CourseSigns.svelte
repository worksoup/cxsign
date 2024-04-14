<script lang="ts">
  import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
  import * as Table from "$lib/components/ui/table/index.js";
  import type { AccountPair } from "./commands/account";
  import type { RawSign } from "./commands/sign";
  import { Page } from "./commands/tools";
  import SignLayout from "./sign/SignLayout.svelte";
  export let signs: RawSign[] = [];
  export let accounts: AccountPair[];
  export let state: Page;
  export let scanning = false;
  let sign: RawSign;
  function onRowClick(s: RawSign) {
    window.history.pushState(
      { state: Page.sign },
      "",
      "?state=1&page=SignLayout"
    );
    sign = s;
    state = Page.sign;
  }
</script>

{#if state == Page.courseSigns}
  <div class="items-center justify-center">
    <ScrollArea class="h-[77.5vh] rounded-md border mb-2">
      <Table.Root>
        <Table.Body>
          {#each signs as sign (sign.name)}
            <Table.Row>
              <Table.Cell
                on:click={() => {
                  onRowClick(sign);
                }}
              >
                <p>
                  {sign.name}
                </p>
              </Table.Cell>
            </Table.Row>
          {/each}
        </Table.Body>
      </Table.Root>
    </ScrollArea>
  </div>
{:else if sign}
  <SignLayout bind:scanning {sign} {accounts} bind:state></SignLayout>
{/if}
