import path from 'path'
import fs from 'fs'
import { tsGenerator } from 'ts-generator'
import { TypeChain } from 'typechain/dist/TypeChain'

async function main() {
  const root = path.join(__dirname, '..', '..')
  const asRepo = path.join(root, 'node_modules/@hoprnet/hopr-ethereum/build/extracted/abis')
  const asLib = path.join(root, '../../../node_modules/@hoprnet/hopr-ethereum/build/extracted/abis')
  const isRepo = fs.existsSync(asRepo)

  let isLib = false
  if (!isRepo) {
    isLib = fs.existsSync(asLib)
  }

  if (!isRepo && !isLib) {
    throw Error("`hopr-ethereum` repo wasn't found")
  }

  await tsGenerator(
    { cwd: root },
    new TypeChain({
      cwd: root,
      rawConfig: {
        files: `${isRepo ? asRepo : asLib}/*.json`,
        outDir: './src/tsc/web3',
        target: 'web3-v1'
      }
    })
  )
}

main().catch(console.error)
