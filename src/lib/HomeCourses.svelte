<script lang="ts">
  import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
  import * as Avatar from "$lib/components/ui/avatar";
  import * as Table from "$lib/components/ui/table/index.js";
  import { type CoursePair } from "$lib/commands/course";
  import CourseSigns from "./CourseSigns.svelte";
  import { listCourseActivities, type RawSign } from "./commands/sign";
  import type { AccountPair } from "./commands/account";
  import { Page } from "./commands/tools";
  import { Button } from "$lib/components/ui/button/index.js";
  import { createEventDispatcher } from "svelte";
  import { Skeleton } from "$lib/components/ui/skeleton/index.js";
  const dispatch = createEventDispatcher();
  export let courses: CoursePair[] = [];
  let signs: RawSign[] = [];
  let accounts: AccountPair[] = [];
  export let updating = false;
  export let state: Page = Page.home;
  export let scanning = false;
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
            {#each courses as course (course.course.name)}
              <Table.Row>
                <Table.Cell
                  on:click={async () => {
                    signs = await listCourseActivities(course.course);
                    signs = signs.sort();
                    accounts = course.unames;
                    state = Page.courseSigns;
                    window.history.pushState(
                      { state: Page.courseSigns },
                      "",
                      "?state=1&page=courseSigns"
                    );
                  }}
                >
                  <div class="flex flex-row items-center space-x-2">
                    <Avatar.Root class="size-7">
                      <Avatar.Image
                        src={course.course.image_url}
                        alt={course.course.name}
                      />
                      <Avatar.Fallback
                        >{course.course.name.at(0)}</Avatar.Fallback
                      >
                    </Avatar.Root>
                    <p class="truncate max-w-[65vw]">
                      {course.course.name}
                    </p>
                  </div>
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
          dispatch("updateCourses");
        }}
      >
        刷新
      </Button>
    </div>
  </div>
{:else}
  <CourseSigns bind:scanning bind:state {signs} {accounts} />
{/if}
