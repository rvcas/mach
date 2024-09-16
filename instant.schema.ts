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
  },
  {},
);

export default graph;
