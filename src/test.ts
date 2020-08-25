import { startServer } from './main'
import Hopr from '@hoprnet/hopr-core'
import { decode } from 'rlp'
import type { HoprOptions } from '@hoprnet/hopr-core'
import { getBootstrapAddresses } from '@hoprnet/hopr-utils'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'

// TODO this should probably be shared between chat and this, and live in a
// utils module.
function parseHosts(): HoprOptions['hosts'] {
  const hosts: HoprOptions['hosts'] = {}
  if (host !== undefined) {
    const str = host.replace(/\/\/.+/, '').trim()
    const params = str.match(/([0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3})\:([0-9]{1,6})/)
    if (params == null || params.length != 3) {
      throw Error(`Invalid IPv4 host. Got ${str}`)
    }

    hosts.ip4 = {
      ip: params[1],
      port: parseInt(params[2]),
    }
  }
  return hosts
}

const network = process.env.HOPR_NETWORK || 'ETHEREUM'
const provider = process.env.HOPR_ETHEREUM_PROVIDER || 'wss://kovan.infura.io/ws/v3/f7240372c1b442a6885ce9bb825ebc36'
const host = process.env.HOPR_HOST || '0.0.0.0:9091' // Default IPv4

function logMessageToNode(msg: Uint8Array) {
  console.log('#### NODE RECEIVED MESSAGE ####')
  try {
    let [decoded, time] = decode(msg) as [Buffer, Buffer]
    console.log('Message:', decoded.toString())
    console.log('Latency:', Date.now() - parseInt(time.toString('hex'), 16) + 'ms')
  } catch (err) {
    console.log('Could not decode message', err)
    console.log(msg.toString())
  }
}

async function main() {
  console.log('Starting...')

  let options: HoprOptions = {
    debug: Boolean(process.env.DEBUG),
    network,
    bootstrapServers: [...(await getBootstrapAddresses()).values()],
    provider,
    hosts: parseHosts(),
    output: logMessageToNode,
    password: process.env.HOPR_PASSWORD || 'switzerland', // TODO!!!
  }

  let NODE: Hopr<HoprCoreConnector>

  NODE = await Hopr.create(options)
  startServer(NODE)
}

main()
