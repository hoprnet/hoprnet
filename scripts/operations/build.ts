import { bash, root } from './utils'
import patch from './patch'
import extractBuild from './extract-build'

type Action = {
  before: string
  cmd: string | (() => Promise<any>)
  after: string
}

const actions: Action[] = [
  { before: 'running solhint on contracts', cmd: `npx solhint ${root}/contracts/**/*.sol`, after: 'done ✅' },
  { before: 'compiling contracts', cmd: 'npx truffle compile', after: 'done ✅' },
  {
    before: "generating contracts' typescript types",
    cmd: `npx typechain --target truffle --outDir ${root}/types/truffle-contracts ${root}/build/contracts/*.json`,
    after: 'done ✅'
  },
  { before: 'applying patches', cmd: patch, after: 'done ✅' },
  { before: 'transpiling typescript files', cmd: 'npx tsc', after: 'done ✅' },
  {
    before: 'extracting compiled output',
    cmd: extractBuild,
    after: 'done ✅'
  }
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
