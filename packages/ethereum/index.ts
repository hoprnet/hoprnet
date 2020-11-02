// require('ts-node/register')
import runner from './utils/runner'

export async function compile(args: string = '') {
  await runner(`yarn build:sol${args ? ' ' + args : ''}`)
}

export async function migrate(args: string = '') {
  await runner(`yarn migrate${args ? ' ' + args : ''}`)
}

export async function fund(args: string = '') {
  await runner(`yarn fund${args ? ' ' + args : ''}`)
}
