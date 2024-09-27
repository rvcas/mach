<script lang="ts">
  import { db } from '$lib/instant';
  import {
    type InstantQueryResult,
    type LifecycleSubscriptionState,
    type User,
    id,
  } from '@instantdb/core';
  import { onMount } from 'svelte';

  import { teamState } from '$lib/team.svelte';
  import { userAuthState } from '$lib/user.svelte';
  import { goto } from '$app/navigation';

  type Teams = InstantQueryResult<typeof db, { teams: {} }>['teams'];

  let teams: Teams = $state([]);
  let defaultTeamName = $state('');
  let teamName = $state('');

  const selectedTeamState = teamState();
  const user = userAuthState();

  let missingDefaultTeam: boolean = $derived(
    user.authState !== undefined && teams.length === 0,
  );

  onMount(() => {
    const unsub = db.subscribeAuth((auth) => {
      if (auth.user) {
        user.setAuthState(auth.user);
      }
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

  async function makeDefaultTeam(e: Event) {
    e.preventDefault();

    if (user.authState) {
      try {
        const teamId = id();
        const membershipsId = id();

        // Create default team for new user
        const result = await db.transact([
          db.tx.teams[teamId].update({
            creatorId: user.authState.id,
            isDefault: true,
            name: defaultTeamName,
          }),

          db.tx.memberships[membershipsId].update({
            teamId,
            userEmail: user.authState.email,
            userId: user.authState.id,
          }),

          db.tx.memberships[membershipsId].link({ teams: teamId }),
        ]);

        console.log(result);
      } catch (e) {
        console.log(e);
      }
    }
  }

  async function makeTeam(e: Event) {
    e.preventDefault();

    if (user.authState) {
      try {
        const teamId = id();
        const membershipsId = id();

        // Create default team for new user
        const result = await db.transact([
          db.tx.teams[teamId].update({
            creatorId: user.authState.id,
            isDefault: false,
            name: teamName,
          }),

          db.tx.memberships[membershipsId].update({
            teamId,
            userEmail: user.authState.email,
            userId: user.authState.id,
          }),

          db.tx.memberships[membershipsId].link({ teams: teamId }),
        ]);

        console.log(result);
      } catch (e) {
        console.log(e);
      }
    }
  }

  async function logout() {
    await db.auth.signOut();
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
        placeholder="Default Team Name"
      />

      <button
        type="submit"
        class="bg-purple-700 text-cyan-100 p-2 rounded-lg shadow-md hover:bg-purple-600 transition-transform transform-gpu hover:translate-y-[-2px]"
      >
        Create
      </button>
    </form>
  </div>
{:else}
  <div class="grid grid-cols-1 gap-4">
    <h1 class="text-2xl">Teams</h1>
    <button
      type="button"
      onclick={() => {
        logout();
      }}
    >
      LOG OUT
    </button>

    <form onsubmit={makeTeam} class="flex flex-col gap-8">
      <input
        type="text"
        class="border rounded p-2 focus:border-red-400 focus:outline-none focus:border-2 shadow-inner"
        bind:value={teamName}
        placeholder="New Team Name"
      />

      <button
        type="submit"
        class="bg-purple-700 text-cyan-100 p-2 rounded-lg shadow-md hover:bg-purple-600 transition-transform transform-gpu hover:translate-y-[-2px]"
      >
        Create
      </button>
    </form>

    <button
      type="button"
      onclick={() => {
        goto('/invites');
      }}
    >
      View Invites
    </button>

    {#each teams as team}
      <div class="p-8 flex flex-col gap-4">
        <button
          type="button"
          onclick={() => {
            selectedTeamState.setTeamId(team.id);
            selectedTeamState.setTeamName(team.name);

            goto(`/team`);
          }}
        >
          {team.id},{team.name},{team.isDefault}
        </button>
        <button
          type="button"
          onclick={() => {
            selectedTeamState.setTeamId(team.id);
            selectedTeamState.setTeamName(team.name);

            goto(`/todos`);
          }}
        >
          TODOS
        </button>
      </div>
    {/each}
  </div>
{/if}
