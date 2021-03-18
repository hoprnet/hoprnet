/*
  Primarely used by `core-ethereum` tests
  @TODO: remove this once `core-ethereum` is refactored
*/
import runner from './tasks/utils/runner'

export async function compile(args: string = '') {
  await runner(`yarn build:sol${args ? ' ' + args : ''}`)
}

export async function migrate(args: string = '') {
  await runner(`yarn migrate${args ? ' ' + args : ''}`)
}

export async function fund(args: string = '') {
  await runner(`yarn fund${args ? ' ' + args : ''}`)
}

export * from './chain'
