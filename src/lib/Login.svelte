<script lang="ts">
  import cxsignLogo from "$lib/icons/cxsign.svg";
  import { Button } from "$lib/components/ui/button";
  import { addAccount, isStringEmpty } from "./commands/account";
  import { Input } from "$lib/components/ui/input/index.js";
  import { createEventDispatcher } from "svelte";
  let uname = "";
  let pwd = "";
  const dispatch = createEventDispatcher();
  export let fristLogin = false;
  let btnDisable = true;
  async function toggleBtn() {
    btnDisable =
      isStringEmpty(uname) ||
      isStringEmpty(pwd) ||
      pwd.length < 8 ||
      pwd.length > 16;
  }
  let errorMsg = "";
  let loginOk = false;
  async function onSubmit() {
    btnDisable = true;
    await addAccount(uname, pwd).then((r) => {
      if (r.isOk) {
        pwd = "";
        loginOk = true;
        uname = "";
        errorMsg = "";
        if (!fristLogin) {
          window.history.back();
        }
        dispatch("login");
      } else {
        errorMsg = r.errMsg;
        btnDisable = false;
      }
    });
  }
</script>

<div class="flex-col items-center justify-center max-w-70">
  <div class="flex justify-center items-center">
    <a
      href="https://github.com/worksoup/cxsign"
      target="_blank"
      class="flex items-center"
    >
      <img
        src={cxsignLogo}
        class="cxsign logo mb-6 flex-row"
        alt="Welcome to Cxsign!"
      />
    </a>
  </div>
  <form on:submit|preventDefault={onSubmit}>
    <div class="grid w-full max-w-sm items-center">
      <Input
        class="mb-6"
        type="tel"
        id="name-input"
        name="uname"
        inputmode="numeric"
        placeholder="手机号"
        bind:value={uname}
        on:input={toggleBtn}
      />
      <Input
        class="mb-6"
        id="pwd-input"
        type="password"
        name="pwd"
        placeholder="密码"
        bind:value={pwd}
        on:input={toggleBtn}
      />
      <div class="flex justify-center items-center">
        {#if errorMsg != ""}
          <p class="text-sm text-red-600">
            {errorMsg}
          </p>
        {:else if loginOk}
          <p class="text-sm text-green-600">登录成功</p>
        {/if}
      </div>
    </div>
    <div class="flex justify-center">
      <Button type="submit" disabled={btnDisable}>登录</Button>
    </div>
  </form>
</div>

<!-- <style>
  .logo.cxsign:hover {
    filter: drop-shadow(0 0 2em #e9987d);
  }
</style> -->
