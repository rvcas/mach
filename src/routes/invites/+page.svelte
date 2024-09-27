<script lang="ts">
  import { goto } from '$app/navigation';
  import { db } from '$lib/instant';
  import { type InstantQueryResult, id } from '@instantdb/core';
  import { onMount } from 'svelte';
  import { userAuthState } from '$lib/user.svelte';

  type Invites = InstantQueryResult<
    typeof db,
    { invites: { teams: {} } }
  >['invites'];

  const user = userAuthState();

  let invites: Invites = $state([]);
  type Invite = (typeof invites)[0];

  onMount(() => {
    if (user.authState) {
      const unsub = db.subscribeQuery(
        {
          invites: {
            teams: {},
            $: { where: { userEmail: user.authState.email } },
          },
        },
        (resp) => {
          if (resp.data) {
            invites = resp.data.invites;
          }
        },
      );

      return () => {
        unsub();
      };
    } else {
      return () => {};
    }
  });

  async function acceptInvite(invite: Invite) {
    console.log(invite);
    if (user.authState && invite.teams) {
      try {
        const memberId = id();

        const result = await db.transact([
          db.tx.memberships[memberId].update({
            teamId: invite.teams.id,
            userEmail: user.authState.email,
            userId: user.authState.id,
          }),
          db.tx.memberships[memberId].link({ teams: invite.teams.id }),
        ]);

        const result2 = await db.transact(db.tx.invites[invite.id].delete());

        console.log(result);
        console.log(result2);
      } catch (e) {
        console.log(e);
      }
    }
  }
</script>

<div>
  <h1>Teams you are invited to</h1>

  <button
    type="button"
    onclick={() => {
      goto('/');
    }}
  >
    Back
  </button>

  {#each invites as invite}
    <div class="p-8 flex flex-col gap-4">
      <p>{invite.teamName}</p>
      <button
        type="button"
        onclick={() => {
          acceptInvite(invite);
        }}
      >
        Join
      </button>
    </div>
  {/each}
</div>
