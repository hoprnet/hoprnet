import { Ganache } from '@hoprnet/hopr-testing'
import addresses from '@hoprnet/hopr-ethereum/lib/chain/addresses'
import { compile, migrate, fund } from '@hoprnet/hopr-ethereum'

let ganache

export const mochaGlobalSetup = async () => {
  ganache = new Ganache()
  await ganache.start()

  await compile()
  await migrate()
  await fund(`--address ${addresses?.localhost?.HoprToken} --accounts-to-fund 4`)
}

export const mochaGlobalTeardown = async () => {
  await ganache.stop()
}
