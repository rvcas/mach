<script lang="ts">
  import { goto } from '$app/navigation';
  import { db } from '$lib/instant';
  import { teamState } from '$lib/team.svelte';
  import { type InstantQueryResult, type User, id } from '@instantdb/core';
  import { onMount } from 'svelte';

  type Todos = InstantQueryResult<typeof db, { todos: {} }>['todos'];
  type Todo = Todos[0];

  const selectedTeamState = teamState();

  let todos: Todos = $state([]);
  let todoText: string = $state('');
  let todoDate: number | undefined = $state();

  onMount(() => {
    const unsubQuery = db.subscribeQuery(
      {
        todos: {
          $: { where: { teams: selectedTeamState.teamId, done: false } },
        },
      },
      (resp) => {
        if (resp.data) {
          todos = resp.data.todos;
        }
      },
    );

    return () => {
      unsubQuery();
    };
  });

  async function createTodo(e: Event) {
    e.preventDefault();

    try {
      const todoId = id();

      const result = await db.transact([
        db.tx.todos[todoId].update({
          text: todoText,
          done: false,
          date: todoDate || Date.now(),
        }),
        db.tx.todos[todoId].link({
          teams: selectedTeamState.teamId,
        }),
      ]);

      console.log(result);
    } catch (e) {
      console.log(e);
    }
  }

  async function finishTodo(t: Todo) {
    try {
      const result = await db.transact([
        db.tx.todos[t.id].update({
          done: true,
        }),
      ]);

      console.log(result);
    } catch (e) {
      console.log(e);
    }
  }
</script>

<div>
  <h1>TODOs for Team: {selectedTeamState.teamName}</h1>

  <button
    type="button"
    onclick={() => {
      goto('/');
    }}
  >
    Back
  </button>

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
    >
      Create
    </button>
  </form>

  {#each todos as todo}
    <div class="p-8 flex flex-col gap-4">
      <button
        type="button"
        onclick={() => {
          finishTodo(todo);
        }}
      >
        {todo.text},{todo.date},{todo.done}
      </button>
    </div>
  {/each}
</div>
