import path from 'path'
import fs from 'fs'
import { tsGenerator } from 'ts-generator'
import { TypeChain } from 'typechain/dist/TypeChain'

async function main() {
  const cwd = path.join(__dirname, '..')
  const asRepo = path.join(cwd, 'node_modules/@hoprnet/hopr-ethereum/build/extracted/abis')
  const asLib = path.join(cwd, '../../../node_modules/@hoprnet/hopr-ethereum/build/extracted/abis')
  const isRepo = fs.existsSync(asRepo)

  let isLib = false
  if (!isRepo) {
    isLib = fs.existsSync(asLib)
  }

  if (!isRepo && !isLib) {
    throw Error("`hopr-ethereum` repo wasn't found")
  }

  await tsGenerator(
    { cwd },
    new TypeChain({
      cwd,
      rawConfig: {
        files: `${isRepo ? asRepo : asLib}/*.json`,
        outDir: './src/tsc/web3',
        target: 'web3-v1'
      }
    })
  )
}

main().catch(console.error)
