<script>
  import '../app.css';

  import { goto } from '$app/navigation';
  import { db } from '$lib/instant';
  import { onMount } from 'svelte';

  onMount(() => {
    const unsub = db.subscribeAuth((auth) => {
      if (!auth.user) {
        goto('/login');
      }
    });

    return unsub;
  });
</script>

<slot></slot>
