#!/usr/bin/env -S yarn ts-node --transpile-only

import Hopr from '@hoprnet/hopr-core'
import type { HoprOptions } from '@hoprnet/hopr-core'
import type { AccountEntry } from '@hoprnet/hopr-utils'
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

  const announcedNodes = (await node.getAnnouncingNodes()).filter((entry: AccountEntry) => entry.hasAnnounced())

  const results: { [key: string]: number } = {}

  await Promise.all(
    announcedNodes.map(async (entry: AccountEntry) => {
      const id = entry.getPeerId()
      const pingResult = await node.ping(entry.getPeerId())

      Object.assign(results, {
        [id.toB58String()]: pingResult.latency
      })
    })
  )

  console.log(results)

  await node.stop()

  process.exit()
}

main().catch(console.log)
