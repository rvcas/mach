import { i } from '@instantdb/core';

export const APP_ID = '6a68d747-241c-43e0-985e-a19d40338716';

const graph = i.graph(
  APP_ID,
  {
    todos: i.entity({
      text: i.string(),
      done: i.boolean(),
      date: i.number().optional(),
      listId: i.string().optional(),
      groupId: i.string().optional(),
      assigneeId: i.string().optional(),
    }),
    lists: i.entity({
      name: i.string(),
    }),
    groups: i.entity({
      name: i.string(),
      date: i.number().optional(),
      listId: i.string().optional(),
    }),
    drawings: i.entity({
      name: i.string(),
      state: i.json(),
      createdAt: i.number(),
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
      },
    },
    // list has many todos
    todosLists: {
      forward: {
        on: 'todos',
        has: 'one',
        label: 'lists',
      },
      reverse: {
        on: 'lists',
        has: 'many',
        label: 'todos',
      },
    },
    // group has many todos
    todosGroups: {
      forward: {
        on: 'todos',
        has: 'one',
        label: 'groups',
      },
      reverse: {
        on: 'groups',
        has: 'many',
        label: 'todos',
      },
    },
    // group has many todos
    groupsLists: {
      forward: {
        on: 'groups',
        has: 'one',
        label: 'lists',
      },
      reverse: {
        on: 'lists',
        has: 'many',
        label: 'groups',
      },
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
        on: 'invites',
        has: 'one',
        label: 'teams',
      },
      reverse: {
        on: 'teams',
        has: 'many',
        label: 'invites',
      },
    },
    // team has many memberships
    membershipsTeams: {
      forward: {
        on: 'memberships',
        has: 'one',
        label: 'teams',
      },
      reverse: {
        on: 'teams',
        has: 'many',
        label: 'memberships',
      },
    },
  },
);

export default graph;
