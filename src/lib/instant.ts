import { id, init_experimental } from '@instantdb/core';

import schema, { APP_ID } from '../../instant.schema';

export const db = init_experimental({ appId: APP_ID, schema });

export function addTodo(text: string) {
  return db.tx.todos[id()].update({
    text,
    done: false,
    createdAt: Date.now(),
  });
}
