/*
  patch types: https://github.com/ethereum-ts/TypeChain/issues/193
*/
import { promisify } from 'util'
import { join } from 'path'
import { readFile, writeFile } from 'fs'
import { root } from '../utils'

const typesFile = join(root, 'types', 'truffle-contracts', 'index.d.ts')

export default async () => {
  const data = await promisify(readFile)(typesFile, 'utf8')

  const result = data.replace(/BigNumber/g, 'BN').replace('import { BN } from "bignumber.js";', '')

  await promisify(writeFile)(typesFile, result, 'utf8')
  console.log('Successfully patched typechain library!')
}
