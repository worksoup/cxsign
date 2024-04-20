<script lang="ts">
  import { Input } from "$lib/components/ui/input/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import { emit } from "@tauri-apps/api/event";
  import {
    canUseCam,
    canUseCap,
    getQrCodeTypeCount,
    Page,
  } from "$lib/commands/tools";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as RadioGroup from "$lib/components/ui/radio-group/index.js";
  import {
    checkPermissions,
    requestPermissions,
    scan,
  } from "@tauri-apps/plugin-barcode-scanner";
  export let scanning: boolean = false;
  export let state: Page = Page.sign;
  let locationStr: string = "";
  const useCam = canUseCam();
  const useCap = canUseCap();
  const qrCodeGetterCount = getQrCodeTypeCount();
  let getQrCodeType: "scan" | "cap" = "scan";
  if (useCam) {
    getQrCodeType = "scan";
  } else {
    getQrCodeType = "cap";
  }
  $: emit("sign:qrcode:location", {
    location_str: locationStr,
  }).then();
  async function qrCodeSign() {
    if (getQrCodeType == "scan") {
      let enc = await scanQrCode();
      enc ? await emit("sign:qrcode:enc", enc) : {};
    } else if (getQrCodeType == "cap") {
      await emit("sign:qrcode:enc", "");
    }
  }
  async function scanQrCode(): Promise<string> {
    let perm = await checkPermissions();
    if (perm == "prompt" || perm == "denied" || perm == null) {
      perm = await requestPermissions();
    }
    if (perm == "granted") {
      scanning = true;
      state = Page.qrCodeScanner;
      window.history.pushState(
        { state: Page.qrCodeScanner },
        "",
        "?state=1&page=QRCODESCAN"
      );
      let scanned = await scan();
      if (scanned) {
        const url = new URL(scanned.content);
        let params = url.searchParams;
        window.history.back();
        return params.get("enc");
      }
    } else return null;
  }
</script>

<div class="flex-col space-y-2">
  <Input bind:value={locationStr} inputmode="text" placeholder="位置" />
  <div class="flex items-center space-x-2">
    {#if qrCodeGetterCount > 1}
      <RadioGroup.Root bind:value={getQrCodeType}>
        <div class="flex items-center space-x-2">
          {#if useCam}
            <RadioGroup.Item value="scan" id="r2" />
            <Label for="r2">扫码</Label>
          {/if}
          {#if useCap}
            <RadioGroup.Item value="cap" id="r3" />
            <Label for="r3">截屏</Label>
          {/if}
        </div>
        <RadioGroup.Input name="spacing" />
      </RadioGroup.Root>
    {/if}
    <Button
      disabled={qrCodeGetterCount == 0}
      on:click={async () => {
        await qrCodeSign();
      }}
    >
      签到
    </Button>
  </div>
</div>
