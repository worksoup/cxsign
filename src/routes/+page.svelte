<script lang="ts">
  import {
    hasAccounts,
    listAccounts,
    loadAccounts,
    type AccountPair,
  } from "$lib/commands/account";
  import * as Tabs from "$lib/components/ui/tabs";
  import {
    type CoursePair,
    loadCourses,
    listCourses,
  } from "$lib/commands/course";
  import HomeSigns from "$lib/HomeSigns.svelte";
  import { listAllActivities, type RawSignPair } from "$lib/commands/sign";
  import HomeCourses from "$lib/HomeCourses.svelte";
  import { Page } from "$lib/commands/tools";
  import { cancel } from "@tauri-apps/plugin-barcode-scanner";
  import HomeUsers from "$lib/HomeUsers.svelte";
  import Login from "$lib/Login.svelte";
  let state: Page = Page.home;
  let coursesFirstClick: boolean = true;
  let courses: CoursePair[] = [];
  let accounts: AccountPair[] = [];
  let signs: RawSignPair[] = [];
  let scanning: boolean = false;
  let unames = new Set<string>();
  let updateing = [true, true, true];
  window.history.replaceState({ state: Page.home }, "");
  window.onpopstate = (ev: { state: { state: Page } }) => {
    let s = ev.state.state;
    console.log(s);
    if (scanning) {
      cancel().then();
    }
    state = s;
  };
  hasAccounts().then(async (data) => {
    let hasAccounts = data;
    if (!hasAccounts) {
      state = Page.login;
      window.history.replaceState(
        { state: Page.login },
        "",
        "?state=1&page=Login"
      );
    } else {
      await loadAccounts();
      signs = await listAllActivities();
      updateing[1] = false;
    }
  });
  async function updateCourses() {
    updateing[0] = true;
    await loadCourses();
    coursesFirstClick = false;
    courses = (await listCourses()).sort((a, b) => {
      return a.course.name.localeCompare(b.course.name);
    });
    updateing[0] = false;
  }
  async function updateSigns() {
    updateing[1] = true;
    signs = await listAllActivities();
    updateing[1] = false;
  }
  async function updateAccounts() {
    updateing[2] = true;
    await loadAccounts();
    await listAccounts()
      .then((data) => {
        accounts = data;
      })
      .catch((error) => {
        console.error(error);
      });
    updateing[2] = false;
  }
</script>

{#if state != Page.login}
  <Tabs.Root value="home">
    <div class="h-[85vh]">
      <Tabs.Content value="course">
        <HomeCourses
          bind:state
          bind:scanning
          bind:updating={updateing[0]}
          {courses}
          on:updateCourses={updateCourses}
        />
      </Tabs.Content>
      <Tabs.Content value="home">
        <HomeSigns
          bind:state
          bind:scanning
          bind:updating={updateing[1]}
          {signs}
          on:updateSigns={updateSigns}
        />
      </Tabs.Content>
      <Tabs.Content value="user">
        <HomeUsers
          bind:state
          bind:accounts
          bind:unames
          bind:updating={updateing[2]}
          on:updateAccounts={updateAccounts}
        />
      </Tabs.Content>
    </div>
    {#if state == Page.home}
      <div class="flex justify-center">
        <Tabs.List>
          <Tabs.Trigger
            value="course"
            on:click={async () => {
              if (coursesFirstClick) {
                await loadCourses();
                coursesFirstClick = false;
                updateing[0] = false;
              }
              courses = (await listCourses()).sort((a, b) => {
                return a.course.name.localeCompare(b.course.name);
              });
            }}
          >
            课程
          </Tabs.Trigger>
          <Tabs.Trigger value="home">主页</Tabs.Trigger>
          <Tabs.Trigger
            value="user"
            on:click={async () => {
              await listAccounts()
                .then((data) => {
                  accounts = data;
                  updateing[2] = false;
                })
                .catch((error) => {
                  console.error(error);
                });
            }}
          >
            用户
          </Tabs.Trigger>
        </Tabs.List>
      </div>
    {/if}
  </Tabs.Root>
{:else}
  <Login
    on:login={async () => {
      await updateAccounts();
      await updateSigns();
      await updateCourses();
    }}
  />
{/if}
