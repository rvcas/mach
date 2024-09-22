<script lang="ts">
  import { Todo, db, type Thing } from '$lib/instant';
  import {
    type InstantQueryResult,
    type LifecycleSubscriptionState,
    type User,
    id,
  } from '@instantdb/core';
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  type Teams = InstantQueryResult<typeof db, { teams: {} }>['teams'];

  let name = $state('');
  let greetMsg = $state('');
  let user: User | undefined = $state();
  let currentTeam: Teams[0] | undefined = $state();
  let teams: Teams = $state([]);
  let todos: { id: string }[] = $state([]);
  let defaultTeamName = $state('');

  let missingDefaultTeam: boolean = $derived(
    user !== undefined && teams.length === 0,
  );

  onMount(() => {
    const unsub = db.subscribeAuth((auth) => {
      user = auth.user;
    });

    const unsubQuery = db.subscribeQuery({ teams: {} }, (resp) => {
      if (resp.data) {
        teams = resp.data.teams;
      }
    });

    return () => {
      unsub();
      unsubQuery();
    };
  });

  async function greet(e: Event) {
    e.preventDefault();

    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    greetMsg = await invoke('greet', { name });

    Todo.create(name);
  }

  async function makeDefaultTeam(e: Event) {
    e.preventDefault();

    if (user) {
      try {
        const teamId = id();
        const membershipsId = id();

        // Create default team for new user
        const result = await db.transact([
          db.tx.teams[teamId].update({
            creatorId: user!.id,
            isDefault: true,
            name: 'default',
          }),

          db.tx.memberships[membershipsId].update({
            teamId,
            userEmail: user.email,
            userId: user.id,
          }),

          db.tx.memberships[membershipsId].link({ teams: teamId }),
        ]);

        console.log(result);
      } catch (e) {
        console.log(e);
      }
    }
  }
</script>

{#if missingDefaultTeam}
  <div class="flex flex-col w-full h-screen items-center justify-center gap-12">
    <h1 class="text-2xl">Welcome to Mach!</h1>

    <p class="text-lg">To get started enter your default team name</p>

    <form onsubmit={makeDefaultTeam} class="flex flex-col gap-8">
      <input
        type="text"
        class="border rounded p-2 focus:border-red-400 focus:outline-none focus:border-2 shadow-inner"
        bind:value={defaultTeamName}
        placeholder=" Default Team Name"
      />

      <button
        type="submit"
        class="bg-purple-700 text-cyan-100 p-2 rounded-lg shadow-md hover:bg-purple-600 transition-transform transform-gpu hover:translate-y-[-2px]"
        >Create</button
      >
    </form>
  </div>
{:else}
  <div class="grid grid-cols-3 gap-4">
    {#each teams as team}
      <div class="p-8 flex flex-col gap-4">
        <p>{team.id}</p>
        <p>{team.name}</p>
        <p>{team.isDefault}</p>
      </div>
    {/each}
  </div>
{/if}
