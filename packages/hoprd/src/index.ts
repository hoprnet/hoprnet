import path from 'path'
import retimer from 'retimer'
import { rmSync } from 'fs'

import {
  get_package_version,
  create_histogram_with_buckets,
  debug,
  MessageInbox,
  HoprKeys,
  Hopr,
  IdentityOptions,
  ApplicationData,
  HalfKeyChallenge
} from '@hoprnet/hopr-utils'

import {
  parse_cli_arguments,
  fetch_configuration,
  to_hoprlib_config,
  parse_private_key,
  HoprdConfig,
  type Api,
  type CliArgs,
  hoprd_hoprd_initialize_crate,
  HoprdPersistentDatabase,
  HoprLibConfig
} from '../lib/hoprd_hoprd.js'
hoprd_hoprd_initialize_crate()

import type { State } from './types.js'
import setupAPI from './api/index.js'

import { decodeMessage } from './api/utils.js'
import { RPCH_MESSAGE_REGEXP } from './api/v3.js'

import EventEmitter from 'events'

const log = debug('hoprd')

/*
 * General HoprMessageEmitter object responsible for emitting
 * data that would otherwise be obtained from the core-transport
 * stream.
 *
 * Ingress part of the chain.
 */
export class WasmHoprMessageEmitter extends EventEmitter {
  public constructor() {
    super()
  }

  public delegate_on(event: string, callback: () => void) {
    this.on(event, callback)
  }
}

const ONBOARDING_INFORMATION_INTERVAL = 30000 // show information every 30sec

export function removeAllInPath(target: string) {
  const p = path.normalize(target)
  rmSync(p, { recursive: true, force: true })
}

function stopGracefully(signal) {
  log(`Process exiting with signal ${signal}`)
  process.exit()
}

process.on('uncaughtExceptionMonitor', (err, origin) => {
  // Make sure we get a log.
  log(`FATAL ERROR, exiting with uncaught exception`, origin, err)
})

process.once('exit', stopGracefully)
process.on('SIGINT', stopGracefully)
process.on('SIGTERM', stopGracefully)

// Metrics
const metric_latency = create_histogram_with_buckets(
  'hoprd_histogram_message_latency_ms',
  'Histogram of measured received message latencies',
  new Float64Array([10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0, 10000.0, 20000.0])
)

// reading the version manually to ensure the path is read correctly
const packageFile = path.normalize(new URL('../package.json', import.meta.url).pathname)
const version = get_package_version(packageFile)
const on_dappnode = (process.env.DAPPNODE ?? 'false').toLowerCase() === 'true'

// Parse the CLI arguments and return the processed object.
// This function may exit the calling process entirely if an error is
// encountered or the version or help are rendered.
function parseCliArguments(args: string[]) {
  let argv: CliArgs
  try {
    argv = parse_cli_arguments(args, process.env) as CliArgs
  } catch (err) {
    // both --version and --help are treated as errors, therefore we need some
    // special handling here to be able to return exit code 0 in such cases
    const message = err instanceof Error ? err.message : (err as string)
    if (message.startsWith('hoprd') || message.startsWith('HOPRd')) {
      console.log(err)
      process.exit(0)
    }
    console.error(err)
    process.exit(1)
  }

  return argv
}

async function main() {
  // Starting with Node.js 15, undhandled promise rejections terminate the
  // process with a non-zero exit code, which makes debugging quite difficult.
  // Therefore adding a promise rejection handler to make sure that the origin of
  // the rejected promise can be detected.
  // addUnhandledPromiseRejectionHandler()
  // Increase the default maximum number of event listeners
  ;(await import('events')).EventEmitter.defaultMaxListeners = 20

  let inbox: MessageInbox
  let state: State = {
    settings: {
      includeRecipient: false
    }
  }

  const setState = (newState: State): void => {
    state = newState
  }

  const getState = (): State => {
    return state
  }

  const argv = parseCliArguments(process.argv.slice(1))
  let cfg: HoprdConfig
  try {
    cfg = fetch_configuration(argv as CliArgs) as HoprdConfig
    console.log('Node configuration: ' + cfg.as_redacted_string())
  } catch (err) {
    console.error(err)
    process.exit(1)
  }

  if (argv.dry_run) {
    process.exit(0)
  }

  try {
    log(`This is HOPRd version ${version}`)

    if (on_dappnode) {
      log('This node appears to be running on an Dappnode')
    }

    // 1. Find or create an identity
    const parsed_private_key =
      cfg.identity.private_key === undefined ? undefined : parse_private_key(cfg.identity.private_key)
    const identity_opts = new IdentityOptions(
      cfg.hopr.db.initialize,
      cfg.identity.file,
      cfg.identity.password,
      cfg.test.use_weak_crypto,
      parsed_private_key
    )

    const keypair = HoprKeys.init(identity_opts)

    log(
      `This node '${keypair.packet_key.public().to_peerid_str()} uses blockchain address '${keypair.chain_key
        .public()
        .to_hex(true)}''`
    )
    log(`chain_key ${keypair.chain_key.public().to_hex(true)}`)
    log(`packet_key ${keypair.packet_key.public().to_peerid_str()}`)

    if (keypair.packet_key.public().to_string().startsWith('0xff')) {
      log(
        `WARNING: this node uses invalid packet key type and won't be able to become an effective relay, please create a new identity!`
      )
    }

    inbox = new MessageInbox(cfg.inbox)

    const logMessageToNode = async (data: ApplicationData) => {
      log(`#### NODE RECEIVED MESSAGE [${new Date().toISOString()}] ####`)
      try {
        let decodedMsg = decodeMessage(data.plain_text)
        log(`Message: ${decodedMsg.msg}`)
        log(`App tag: ${data.application_tag ?? 0}`)
        log(`Latency: ${decodedMsg.latency} ms`)
        metric_latency.observe(decodedMsg.latency)

        if (RPCH_MESSAGE_REGEXP.test(decodedMsg.msg)) {
          log(`RPCh: received message [${decodedMsg.msg}]`)
        }

        // Needs to be created new, because the incoming `data` is from serde_wasmbindgen and not a Rust WASM object
        // Use the plain message, not the RLP encoded form. At this point we don't
        // care about the latency anymore.
        let appData = new ApplicationData(data.application_tag, new TextEncoder().encode(decodedMsg.msg))
        await inbox.push(appData)
      } catch (err) {
        log(
          'Could not decode message',
          err instanceof Error ? err.message : 'Unknown error',
          data.plain_text.toString()
        )
      }
    }

    // 2. Create node instance
    log('Creating HOPR Node')
    // TODO: originally (DAPPNODE support) the safe and module address could have been undefined to allow safe setup
    // if safe address or module address is not provided, replace with values stored in the db
    const hoprlib_cfg: HoprLibConfig = to_hoprlib_config(cfg)

    // NODE Rust to TS hacked setup before fully migrating everything
    let message_emitter = new WasmHoprMessageEmitter()
    const onAcknowledgement = (ackChallenge: HalfKeyChallenge) => {
      // Can subscribe to both: per specific message or all message acknowledgments
      message_emitter.emit(`hopr:message-acknowledged:${ackChallenge.to_hex()}`)
      message_emitter.emit('hopr:message-acknowledged', ackChallenge.to_hex())
    }

    const onReceivedMessage = (msg: ApplicationData) => {
      message_emitter.emit('hopr:message', msg)
    }

    log('Creating the hopr-lib node instance...')
    let node = new Hopr(
      hoprlib_cfg,
      keypair.packet_key,
      keypair.chain_key,
      message_emitter as WasmHoprMessageEmitter,
      onReceivedMessage,
      onAcknowledgement
    )

    let loops = await node.run()

    // Subscribe to node events
    log('Subscribing incoming messages to inbox')
    node.on('hopr:message', logMessageToNode)

    let api = cfg.api as Api
    console.log(JSON.stringify(api, null, 2))

    log('Creating HOPRd only database (auth...)')
    const hoprd_db_path = path.join(cfg.hopr.db.data, 'db', 'hoprd')
    let hoprdDb = new HoprdPersistentDatabase(hoprd_db_path)

    const startApiListen = setupAPI(
      node,
      inbox,
      { getState, setState },
      {
        disableApiAuthentication: api.is_auth_disabled(),
        apiHost: api.host.address(),
        apiPort: api.host.port,
        apiToken: api.is_auth_disabled() ? null : api.auth_token()
      },
      hoprdDb
    )
    // start API server only if API flag is true
    if (cfg.api.enable) startApiListen()

    showOnboardingInformation(node)

    // Won't return until node is terminated
    await loops.execute()
  } catch (e) {
    log('Node failed to start: ' + e)
    process.exit(1)
  }
}

async function showOnboardingInformation(node: Hopr): Promise<void> {
  let address_original = node.getEthereumAddress()
  let address = address_original.to_string()
  const version = node.getVersion()
  const peer_id = node.peerId()
  const isAllowed = await node.isAllowedToAccessNetwork(peer_id)
  if (isAllowed) {
    const msg = `
      Node information:

      Node peerID: ${peer_id}
      Node address: ${address}
      Node version: ${version}
      `
    log(msg)
    return
  }

  const msg = `
    Node information:

    Node peerID: ${peer_id}
    Node address: ${address}
    Node version: ${version}

    Once you become eligible to join the HOPR network, you can continue your onboarding by using the following URL: https://hub.hoprnet.org/staking/onboarding?HOPRdNodeAddressForOnboarding=${address}, or by manually entering the node address of your node on https://hub.hoprnet.org/.
      `
  log(msg)
  retimer(async () => {
    await showOnboardingInformation(node)
  }, ONBOARDING_INFORMATION_INTERVAL)
}

main()
