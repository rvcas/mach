import { i } from '@instantdb/core';

export const APP_ID = '409c6ca4-bdd4-43e2-b342-ce12cc7ca281';

const graph = i.graph(
  APP_ID,
  {
    todos: i.entity({
      text: i.string(),
      done: i.boolean(),
      createdAt: i.number(),
    }),
    drawings: i.entity({
      name: i.string(),
      state: i.json(),
    }),
    invites: i.entity({
      teamId: i.string(),
      teamName: i.string(),
      userEmail: i.string(),
    }),
    memberships: i.entity({
      teamId: i.string(),
      userEmail: i.string(),
      userId: i.string(),
    }),
    teams: i.entity({
      creatorId: i.string(),
      isDefault: i.boolean(),
      name: i.string(),
    }),
  },
  {
    // team has many todos
    todosTeams: {
      forward: {
        on: 'todos',
        has: 'one',
        label: 'teams',
      },
      reverse: {
        on: 'teams',
        has: 'many',
        label: 'todos',
      }
    },
    // team has many drawings
    drawingsTeams: {
      forward: {
        on: 'drawings',
        has: 'one',
        label: 'teams',
      },
      reverse: {
        on: 'teams',
        has: 'many',
        label: 'drawings',
      },
    },
    // team has many invites
    invitesTeams: {
      forward: {
        on: "invites",
        has: "one",
        label: "teams",
      },
      reverse: {
        on: "teams",
        has: "many",
        label: "invites",
      },
    },
    // team has many memberships
    membershipsTeams: {
      forward: {
        on: "memberships",
        has: "one",
        label: "teams",
      },
      reverse: {
        on: "teams",
        has: "many",
        label: "memberships",
      },
    },
  },
);

export default graph;
