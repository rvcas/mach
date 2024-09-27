<script lang="ts">
  import { goto } from '$app/navigation';
  import { db } from '$lib/instant';
  import { teamState } from '$lib/team.svelte';
  import { type InstantQueryResult, type User, id } from '@instantdb/core';
  import { onMount } from 'svelte';
  import { z } from 'zod';

  type Memberships = InstantQueryResult<
    typeof db,
    { memberships: {} }
  >['memberships'];

  const selectedTeamState = teamState();
  const formSchema = z.string().email({ message: 'Invalid email address' });

  let issue = $state('');
  let inviteeEmail: string = $state('');
  let inviteSent: boolean = $state(false);

  let members: Memberships = $state([]);

  onMount(() => {
    const unsub = db.subscribeQuery(
      { memberships: { $: { where: { teams: selectedTeamState.teamId } } } },
      (resp) => {
        if (resp.data) {
          members = resp.data.memberships;
        }
      },
    );

    return () => {
      unsub();
    };
  });

  async function invite(e: Event) {
    e.preventDefault();

    try {
      const validatedEmail = formSchema.parse(inviteeEmail);

      const inviteId = id();

      const result = await db.transact([
        db.tx.invites[inviteId].update({
          teamId: selectedTeamState.teamId,
          teamName: selectedTeamState.teamName,
          userEmail: validatedEmail,
        }),
        db.tx.invites[inviteId].link({ teams: selectedTeamState.teamId }),
      ]);

      inviteSent = true;
      console.log(result);
    } catch (e) {
      if (e instanceof z.ZodError) {
        issue = e.issues[0].message;
      }
      console.log(e);
    }
  }
</script>

<div>
  <h1>Current Team Members for: {selectedTeamState.teamName}</h1>

  <button
    type="button"
    onclick={() => {
      goto('/');
    }}
  >
    Back
  </button>

  <form onsubmit={invite} class="flex flex-col gap-8">
    <input
      type="email"
      class="border rounded p-2 focus:border-red-400 focus:outline-none focus:border-2 shadow-inner"
      placeholder="me@domain.com"
      bind:value={inviteeEmail}
    />
    {#if issue}
      <p>{issue}</p>
    {/if}

    <button
      type="submit"
      class="bg-purple-700 text-cyan-100 p-2 rounded-lg shadow-md hover:bg-purple-600 transition-transform transform-gpu hover:translate-y-[-2px]"
    >
      Invite
    </button>
    {#if inviteSent}
      <p>Invite Sent!</p>
    {/if}
  </form>

  {#each members as member}
    <div class="p-8 flex flex-col gap-4">
      <p>{member.userId},{member.userEmail}</p>
    </div>
  {/each}
</div>
