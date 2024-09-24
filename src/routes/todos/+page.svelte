<script lang="ts">
  import { goto } from '$app/navigation';
  import { db } from '$lib/instant';
  import { todosTeamState } from '$lib/todos.svelte';
  import { type InstantQueryResult, type User, id } from '@instantdb/core';
  import { onMount } from 'svelte';

  type Todos = InstantQueryResult<typeof db, { todos: {} }>['todos'];

  // let { teamName, teamId }: { teamName: string; teamId: string } = ;

  let user: User | undefined = $state();

  let todos: Todos = $state([]);
  let todoText: string = $state('');
  let todoDate: number | undefined = $state();

  const teamState = todosTeamState();

  onMount(() => {
    const unsub = db.subscribeAuth((auth) => {
      user = auth.user;
    });

    const unsubQuery = db.subscribeQuery(
      { todos: { $: { where: { teams: teamState.teamId } } } },
      (resp) => {
        if (resp.data) {
          todos = resp.data.todos;
        }
      },
    );

    return () => {
      unsub();
      unsubQuery();
    };
  });

  async function createTodo(e: Event) {
    e.preventDefault();

    if (user) {
      try {
        const todoId = id();

        const result = await db.transact([
          db.tx.todos[todoId].update({
            text: todoText,
            done: false,
            date: todoDate || Date.now(),
          }),
          db.tx.todos[todoId].link({
            teams: teamState.teamId,
          }),
        ]);

        console.log(result);
      } catch (e) {
        console.log(e);
      }
    }
  }
</script>

<div>
  <h1>TODOs for Team: {teamState.teamName}</h1>

  <button
    type="button"
    onclick={() => {
      goto('/');
    }}>Back</button
  >

  <form onsubmit={createTodo} class="flex flex-col gap-8">
    <input
      type="text"
      class="border rounded p-2 focus:border-red-400 focus:outline-none focus:border-2 shadow-inner"
      bind:value={todoText}
      placeholder="Write a Todo"
    />

    <input
      type="text"
      class="border rounded p-2 focus:border-red-400 focus:outline-none focus:border-2 shadow-inner"
      bind:value={todoDate}
      placeholder="Todo Due Date"
    />

    <button
      type="submit"
      class="bg-purple-700 text-cyan-100 p-2 rounded-lg shadow-md hover:bg-purple-600 transition-transform transform-gpu hover:translate-y-[-2px]"
      >Create</button
    >
  </form>

  {#each todos as todo}
    <div class="p-8 flex flex-col gap-4">
      <p>{todo.id},{todo.text},{todo.date},{todo.done}</p>
    </div>
  {/each}
</div>
