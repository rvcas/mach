import { id, init_experimental } from '@instantdb/core';

import schema, { APP_ID } from '../../instant.schema';

export const db = init_experimental({ appId: APP_ID, schema });

export const getId = () => id();

export module Todo {
  export function create(text: string) {
    return db.transact(
      db.tx.todos[id()].update({
        text,
        done: false,
        assigneeId: null,
        date: null,
        groupId: null,
        listId: null,
      }),
    );
  }
}

export module Team {
  export function create(
    creatorId: string,
    creatorEmail: string,
    name: string,
    isDefault: boolean,
  ) {
    const teamId = id();
    const membershipId = id();

    return db.transact([
      db.tx.teams[teamId].update({
        name,
        isDefault,
        creatorId,
      }),
      db.tx.memberships[membershipId].update({
        userEmail: creatorEmail,
      }),
      db.tx.memberships[membershipId].link({
        teams: teamId,
      }),
    ]);
  }
}
