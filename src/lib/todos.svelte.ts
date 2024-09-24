let teamName = $state('');
let teamId = $state('');

export function todosTeamState() {
  function setTeamName(name: string) {
    teamName = name;
  }

  function setTeamId(id: string) {
    teamId = id;
  }

  return {
    get teamName() {
      return teamName;
    },
    get teamId() {
      return teamId;
    },
    setTeamName,
    setTeamId,
  };
}
