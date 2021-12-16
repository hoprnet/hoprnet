// import { spawn } from 'child_process'
import { main } from './index'
import { debug, wait } from '@hoprnet/hopr-utils'

const namespace = 'hopr:test:hoprd'
const log = debug(namespace)
//const hardhatLog = debug(`${namespace}:hardhat`)

describe('HOPRd', () => {
  it('should close channels properly between alice and bob', async function () {
    this.timeout(0)
    // @TODO Replace calling hardhat outside of this test for doing it inside to have better control. For now, the complexities around
    // spawning the node within mocha are not worth the troubles. Also, since we still need to copy the `hardhat-localhost` contracts
    // manually (see README.md), this can't be automated yet.
    // log('Starting hardhat')
    // const hardhat = spawn('yarn', ['run:network'], { env: { NODE_ENV: 'development', HOPR_ENVIRONMENT_ID: 'hardhat-localhost' } })
    //   .on('error', function (err) { throw err })
    // hardhat.stdout.on('data', (msg) => {
    //   hardhatLog('Stdout from hardhat:', msg)
    // })
    // log('Hardhat has been started, we can now start alice.')
    log('Starting alice')
    process.argv.push('--password="hi"', `--data="/tmp/${Date.now()}-db"`, '--testUseWeakCrypto', '--init', `--identity="/tmp/${Date.now()}-identity"`, '--environment hardhat-localhost')
    const alice = await main();
    alice.on('hopr:monitoring:start', () => {
      log('Alice has managed to start its chain provider successfully.')
    })
    await wait(10000) // holding on 10secs to let multiple un-awaitable operations complete.
    log('alice has been completed')
  })
})
