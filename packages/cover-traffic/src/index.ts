import Hopr from '@hoprnet/hopr-core'
import type { HoprOptions } from '@hoprnet/hopr-core'

async function main() {
  const peerId = null
  const options: HoprOptions = {} as any
  const node = new Hopr(peerId, options)
  await node.waitForFunds()
  await node.start()
}

main()
