<script lang="ts">
  import { z } from 'zod';

  import { goto } from '$app/navigation';
  import { db } from '$lib/instant';

  let code: string | undefined = $state();
  let email: string | undefined = $state();
  let issue: string | undefined = $state();
  let waitingForCode = $state(false);

  const formSchema = z.string().email({ message: 'Invalid email address' });

  async function submitCode(e: Event) {
    e.preventDefault();

    if (email && code) {
      await db.auth.signInWithMagicCode({ email, code });

      goto('/');
    }
  }

  async function login(e: Event) {
    e.preventDefault();

    try {
      const validatedEmail = formSchema.parse(email);

      await db.auth.sendMagicCode({ email: validatedEmail });

      waitingForCode = true;
    } catch (err) {
      if (err instanceof z.ZodError) {
        issue = err.issues[0].message;
      }
    }
  }

  function reset() {
    waitingForCode = false;
    code = undefined;
  }
</script>

<div>
  <h1>Login</h1>
  {#if waitingForCode}
    <form onsubmit={submitCode}>
      <input type="text" bind:value={code} />

      <button type="submit">Login</button>
    </form>

    <button onclick={reset}>Back</button>
  {:else}
    <form onsubmit={login}>
      <input type="email" placeholder="me@domain.com" bind:value={email} />
      {#if issue}
        <p>{issue}</p>
      {/if}

      <button type="submit">Get Magic Code</button>
    </form>
  {/if}
</div>
