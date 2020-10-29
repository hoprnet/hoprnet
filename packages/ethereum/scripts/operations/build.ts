import extractBuild from './extract-build'
import { bash, root } from './utils'

type Action = {
  before: string
  cmd: string | (() => Promise<any>)
  after: string
}

const SUCCESS_MSG = 'success ✅'

const actions: Action[] = [
  // @TODO: re-enable once solhint supports solidity 0.6
  // { before: 'running solhint on contracts', cmd: `npx solhint ${root}/contracts/**/*.sol`, after: SUCCESS_MSG },
  { before: 'compiling contracts', cmd: 'npx truffle compile', after: SUCCESS_MSG },
  {
    before: "generating contracts' typescript types",
    cmd: `npx typechain --target truffle-v5 --outDir ${root}/types/truffle-contracts ${root}/build/contracts/*.json`,
    after: SUCCESS_MSG,
  },
  {
    before: 'extracting compiled output',
    cmd: extractBuild,
    after: SUCCESS_MSG,
  },
  { before: 'transpiling typescript files', cmd: 'npx tsc', after: SUCCESS_MSG },
]

export default async () => {
  for (const action of actions) {
    console.log(action.before)

    if (typeof action.cmd === 'string') {
      await bash(action.cmd)
    } else {
      await action.cmd()
    }

    console.log(action.after)
  }
}
