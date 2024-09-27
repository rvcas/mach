import { type User } from '@instantdb/core';

let userState: User | undefined = $state();

export function userAuthState() {
  function setAuthState(user: User) {
    userState = user;
  }

  return {
    get authState() {
      return userState;
    },
    setAuthState,
  };
}
