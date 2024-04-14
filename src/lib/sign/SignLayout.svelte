<script lang="ts">
  import Users from "$lib/sign/components/Users.svelte";
  import QrCode from "./QrCode.svelte";
  import Gesture from "./Gesture.svelte";
  import Location from "./Location.svelte";
  import Signcode from "./Signcode.svelte";
  import {
    signSingle,
    type RawSign,
    SignType,
    prepareSign,
    getSignType,
  } from "$lib/commands/sign";
  import { type AccountPair } from "$lib/commands/account";
  import { emit } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";
  import { Page } from "$lib/commands/tools";
  import Other from "./Other.svelte";
  export let sign: RawSign;
  export let accounts: AccountPair[];
  export let state: Page;
  export let scanning: boolean = false;
  let unames = new Set<string>();
  let signType = SignType.normal;
  let userListDisabled = false;
  onMount(async () => {
    await prepareSign(sign, accounts);
    signType = await getSignType();
    await signSingle();
  });
  async function quitSignal() {
    console.log("quit.");
    let type_ = signType as string;
    if (type_ == "qrcode") {
      await emit("sign:qrcode:location", "quit");
      await emit("sign:qrcode:enc", "quit");
    } else {
      await emit("sign:" + type_, "quit");
    }
  }
  async function onSign() {
    userListDisabled = true;
    switch (signType) {
      case SignType.photo:
        await emit("sign:photo");
        break;
      case SignType.normal:
        await emit("sign:normal");
        break;
      case SignType.unknown:
        await emit("sign:unknown");
        break;
      default:
    }
  }
  onDestroy(quitSignal);
</script>

<div class="flex-col space-y-2">
  <Users bind:accounts bind:unames bind:disabled={userListDisabled} />
  {#if signType == SignType.qrcode}
    <QrCode
      bind:state
      bind:scanning
      on:sign={onSign}
    />
  {:else if signType == SignType.gesture}
    <Gesture
      on:sign={() => {
        userListDisabled = true;
      }}
    />
  {:else if signType == SignType.location}
    <Location
      on:sign={() => {
        userListDisabled = true;
      }}
    />
  {:else if signType == SignType.signcode}
    <Signcode
      on:sign={() => {
        userListDisabled = true;
      }}
    />
  {:else}
    <Other on:sign={onSign} />
  {/if}
</div>
