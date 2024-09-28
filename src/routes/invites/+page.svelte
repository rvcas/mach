<script lang="ts">
  import { goto } from '$app/navigation';
  import { db } from '$lib/instant';
  import { type InstantQueryResult, id } from '@instantdb/core';
  import { onMount } from 'svelte';
  import { userAuthState } from '$lib/user.svelte';

  type Invites = InstantQueryResult<typeof db, { invites: {} }>['invites'];
  type Invite = Invites[0];

  const user = userAuthState();

  let invites: Invites = $state([]);

  onMount(() => {
    if (user.authState) {
      const unsub = db.subscribeQuery(
        {
          invites: {
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
    if (user.authState) {
      try {
        const memberId = id();

        const result = await db.transact([
          db.tx.memberships[memberId].update({
            teamId: invite.teamId,
            userEmail: user.authState.email,
            userId: user.authState.id,
          }),
          db.tx.memberships[memberId].link({ teams: invite.teamId }),
        ]);

        const result2 = await db.transact(db.tx.invites[invite.id].delete());

        console.log(result);
        console.log(result2);
      } catch (e) {
        console.log(e);
      }
    }
  }

  async function deleteInvite(invite: Invite) {
    if (user.authState) {
      try {
        const result = await db.transact([db.tx.invites[invite.id].delete()]);

        console.log(result);
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
      <button
        type="button"
        onclick={() => {
          deleteInvite(invite);
        }}
      >
        Delete
      </button>
    </div>
  {/each}
</div>
