#!/usr/bin/env -S yarn --silent ts-node

import Hopr from '@hoprnet/hopr-core'
import type { HoprOptions } from '@hoprnet/hopr-core'
import PeerId from 'peer-id'

import yargs from 'yargs/yargs'

async function main() {
  const argv = yargs(process.argv.slice(2))
    .option('provider', {
      describe: 'example: --provider wss://myprovider.name.org',
      demandOption: true,
      type: 'string'
    })
    .parseSync()

  const opts: HoprOptions = {
    provider: argv.provider,
    disablePersistence: true
  }

  const node = new Hopr(await PeerId.create({ keyType: 'secp256k1' }), opts)

  await node.start()

  node.getAnnouncingNodes()

  // @TODO print nodes
}

main().catch(console.log)
