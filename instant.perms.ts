export default {
  todos: {
    bind: ['isMember', "auth.id in data.ref('teams.memberships.userId')"],
    allow: {
      view: 'isMember',
      create: 'isMember',
      delete: 'isMember',
      update: 'isMember',
    },
  },
  drawings: {
    bind: ['isMember', "auth.id in data.ref('teams.memberships.userId')"],
    allow: {
      view: 'isMember',
      create: 'isMember',
      delete: 'isMember',
      update: 'isMember',
    },
  },
  invites: {
    bind: [
      'isMember',
      "auth.id in data.ref('teams.memberships.userId')",
      'isInvitee',
      'auth.email == data.userEmail',
      'isDefault',
      "data.ref('teams.isDefault')",
    ],
    allow: {
      view: 'isInvitee || isMember',
      create: 'isMember && !isDefault',
      delete: 'isInvitee || isMember',
      update: 'false',
    },
  },
  memberships: {
    bind: [
      'isMember',
      "auth.id in data.ref('teams.memberships.userId')",
      'isDefault',
      "data.ref('teams.isDefault')",
      'isInviteeOrCreator',
      "(auth.id == data.ref('teams.creatorId') && size(data.ref('teams.memberships.userId')) == 1) || auth.email in data.ref('teams.invites.userEmail')",
      'isUser',
      'auth.id == data.userId',
      'isDefault',
      "data.ref('teams.isDefault')",
    ],
    allow: {
      view: 'isMember',
      create: 'isInviteeOrCreator',
      delete: 'isUser && !isDefault',
      update: 'false',
    },
  },
  teams: {
    bind: [
      'isMember',
      "auth.id in data.ref('memberships.userId')",
      'isDefault',
      'data.isDefault',
      'isCreator',
      'auth.id == data.creatorId',
    ],
    allow: {
      view: 'isMember',
      create: 'isMember && isCreator',
      delete: 'isCreator && !isDefault',
      update:
        'isMember && data.creatorId == newData.creatorId && data.isDefault == newData.isDefault',
    },
  },
};
