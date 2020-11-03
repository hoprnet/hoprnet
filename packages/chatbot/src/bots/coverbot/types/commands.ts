export enum AdminSubCommands {
  coverTrafficCycle = 'coverTrafficCycle',
  saveState = 'saveState',
  help = 'help',
}

export enum VerifySubCommands {
  tweet = 'tweet',
  status = 'status',
  help = 'help',
}

export enum StatsSubCommands {
  connected = 'connected',
  help = 'help',
}

export enum BotCommands {
  rules = 'rules',
  help = 'help',
  admin = 'admin',
  verify = 'verify',
  stats = 'stats',
}

export type AvailableSubCommands = VerifySubCommands | StatsSubCommands | AdminSubCommands
export type AvailableCommands = BotCommands | AvailableSubCommands
