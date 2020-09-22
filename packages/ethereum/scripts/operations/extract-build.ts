/*
  extract abi and bytecode into a seperate folder,
  the files then can be used by application by simply
  fetching the files through rawgit or similar
*/
import { join } from 'path'
import { readdir, readFile, writeFile, mkdir } from 'fs'
import { promisify } from 'util'
import { root } from './utils'

type Item = {
  name: string
  value: any
}

const inputPath = join(root, 'build', 'contracts')
const outputPath = join(root, 'build', 'extracted')

const getData = async () => {
  return promisify(readdir)(inputPath)
    .then((list) => {
      return list
        .filter((file) => file.startsWith('Hopr'))
        .map((file) => {
          return join(inputPath, file)
        })
    })
    .then((files) => {
      return Promise.all(
        files.map(async (file) => {
          return promisify(readFile)(file, {
            encoding: 'utf8',
          })
            .then((txt) => JSON.parse(txt))
            .then((json) => {
              return {
                contractName: json.contractName,
                abi: json.abi,
                bytecode: json.bytecode,
              }
            })
        })
      )
    })
}

const writeData = async (folder: string, items: Item[]) => {
  await promisify(mkdir)(join(outputPath, folder), {
    recursive: true,
  })

  return Promise.all(
    items.map((item) => {
      return promisify(writeFile)(join(outputPath, folder, `${item.name}.json`), JSON.stringify(item.value, null, 2))
    })
  )
}

export default async () => {
  const data = await getData()

  const abis = data.map((output) => ({
    name: output.contractName,
    value: output.abi,
  }))

  return writeData('abis', abis)
}
