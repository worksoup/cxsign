<script lang="ts">
  import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
  import * as Avatar from "$lib/components/ui/avatar";
  import * as Table from "$lib/components/ui/table/index.js";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import { Label } from "$lib/components/ui/label";
  import { isStringEmpty, type AccountPair } from "$lib/commands/account";
  import { addUnames, clearUnames } from "$lib/commands/sign";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";
  export let unames: Set<string> = new Set([]);
  export let accounts: AccountPair[] = [];
  export let disabled = false;
  let unlistenSucess: UnlistenFn, unlistenFail: UnlistenFn;
  let resultsFailedCount = 0;
  for (const account of accounts) {
    unames.add(account.uname);
  }
  $: if (unames.size == resultsFailedCount) {
    resultsFailedCount = 0;
    updateUnames().then();
    disabled = false;
  }
  unames = unames;
  addUnames(unames).then();
  async function listenSignResults() {
    let sucess = listen<string>("sign:susses", (e) => {
      let uname = e.payload;
      let p = document.getElementById(
        "sign-result-msg-" + uname
      ) as HTMLParagraphElement;
      p.textContent = "签到成功";
      p.className = "text-right truncate text-green-600";
      removeOrAddElement(uname);
      setTimeout(() => {
        p.textContent = "";
        p.className = "text-right truncate";
      }, 2500);
    });
    let fail = listen<string[]>("sign:fail", (e) => {
      let [uname, msg] = e.payload;
      let p = document.getElementById(
        "sign-result-msg-" + uname
      ) as HTMLParagraphElement;
      p.textContent = msg;
      p.className = "text-right truncate text-red-600";
      resultsFailedCount += 1;
      setTimeout(() => {
        p.textContent = "";
        p.className = "text-right truncate";
      }, 2500);
    });
    [unlistenSucess, unlistenFail] = await Promise.all([sucess, fail]);
  }
  function removeOrAddElement(uname: string) {
    if (unames.has(uname)) {
      unames.delete(uname);
    } else {
      unames.add(uname);
    }
    unames = unames;
  }
  async function updateUnames() {
    await clearUnames();
    await addUnames(unames);
    console.log("update unames");
  }
  onMount(listenSignResults);
  onDestroy(() => {
    unlistenFail();
    unlistenSucess();
  });
</script>

<div class="items-center justify-center">
  <ScrollArea class="h-48 rounded-md border">
    <Table.Root>
      <Table.Body>
        {#each accounts as account (account.name)}
          <div class="flex flex-row items-center space-x-2 ml-4">
            <Checkbox
              {disabled}
              checked={unames.has(account.uname)}
              id={"ulcb-" + account.uname}
              onCheckedChange={async () => {
                removeOrAddElement(account.uname);
                await updateUnames();
              }}
            />
            <Label class="flex grow" for={"ulcb-" + account.uname}>
              <Table.Row>
                <Table.Cell>
                  <div class="flex flex-row items-center space-x-2 grow">
                    <Avatar.Root class="size-7">
                      <Avatar.Image src={account.avatar} alt={account.name} />
                      <Avatar.Fallback>{account.name.at(0)}</Avatar.Fallback>
                    </Avatar.Root>
                    <p>
                      {account.name}
                    </p>
                    <div class="flex flex-row-reverse grow">
                      <p
                        class="text-right truncate"
                        id={"sign-result-msg-" + account.uname}
                      ></p>
                    </div>
                  </div>
                </Table.Cell>
              </Table.Row>
            </Label>
          </div>
        {/each}
      </Table.Body>
    </Table.Root>
  </ScrollArea>
</div>
