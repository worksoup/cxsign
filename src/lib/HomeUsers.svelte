<script lang="ts">
  import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
  import * as Avatar from "$lib/components/ui/avatar";
  import * as Table from "$lib/components/ui/table/index.js";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import { Label } from "$lib/components/ui/label";
  import { deleteAccounts, type AccountPair } from "$lib/commands/account";
  import { Page } from "./commands/tools";
  import { createEventDispatcher } from "svelte";
  import { Skeleton } from "$lib/components/ui/skeleton/index.js";
  const dispatch = createEventDispatcher();
  export let unames = new Set<string>();
  export let accounts: AccountPair[] = [];
  export let state: Page = Page.home;
  export let updating = false;
  let contentDialogOpen = false;
  function unames2Names(): string[] {
    let names: string[] = [];
    for (const uname of unames) {
      names = [
        ...names,
        accounts.find((account) => {
          return account.uname == uname;
        }).name,
      ];
    }
    return names;
  }
  function removeOrAddElement(uname: string) {
    if (unames.has(uname)) {
      unames.delete(uname);
    } else {
      unames.add(uname);
    }
    unames = unames;
  }
  async function confirmDeleteAccount() {
    await deleteAccounts(unames)
      .then(async () => {
        unames.forEach((uname) => {
          let index = accounts.findIndex((account) => {
            return account.uname == uname;
          });
          accounts.splice(index, 1);
          accounts = accounts;
        });
        unames = new Set<string>();
        if (accounts.length == 0) {
          window.location.reload();
        }
      })
      .catch((error) => {
        console.error(error);
      });
    contentDialogOpen = false;
  }
</script>

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
          {#each accounts as account (account.name)}
            <div class="flex flex-row items-center space-x-2 ml-4">
              <Checkbox
                checked={unames.has(account.uname)}
                id={"ulcb-" + account.uname}
                onCheckedChange={async () => {
                  removeOrAddElement(account.uname);
                }}
              />
              <Label class="flex grow" for={"ulcb-" + account.uname}>
                <Table.Row>
                  <Table.Cell>
                    <div class="flex flex-row items-center space-x-2">
                      <Avatar.Root class="size-7">
                        <Avatar.Image src={account.avatar} alt={account.name} />
                        <Avatar.Fallback>{account.name.at(0)}</Avatar.Fallback>
                      </Avatar.Root>
                      <p class="truncate">
                        {account.name}
                      </p>
                    </div>
                  </Table.Cell>
                </Table.Row>
              </Label>
            </div>
          {/each}
        {/if}
      </Table.Body>
    </Table.Root>
  </ScrollArea>

  <div class="flex justify-center">
    <div class="flex grow space-x-2">
      <Button
        variant="destructive"
        class="grow w-1/3"
        disabled={!unames.size}
        on:click={() => {
          console.log("将要删除：", unames);
          contentDialogOpen = true;
        }}
      >
        删除
      </Button>
      <Button
        class="grow w-1/3"
        on:click={() => {
          state = Page.login;
          window.history.pushState(
            { state: Page.login },
            "",
            "?state=1?page=Login"
          );
        }}
      >
        添加
      </Button>
      <Button
        disabled={updating}
        class="grow w-1/3"
        on:click={() => {
          dispatch("updateAccounts");
        }}
      >
        刷新
      </Button>
    </div>
  </div>

  <Dialog.Root bind:open={contentDialogOpen}>
    <Dialog.Content class="sm:max-w-[425px]">
      <Dialog.Header>
        <Dialog.Title>是否删除?</Dialog.Title>
      </Dialog.Header>
      是否删除用户 {unames2Names()} 的账号?
      <Dialog.Footer class="flex-row-reverse">
        <Button
          variant="outline"
          class="ml-4"
          on:click={() => {
            contentDialogOpen = false;
          }}>否</Button
        >
        <Button variant="destructive" on:click={confirmDeleteAccount}>是</Button
        >
      </Dialog.Footer>
    </Dialog.Content>
  </Dialog.Root>
</div>
