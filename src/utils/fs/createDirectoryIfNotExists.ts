import { accessSync, mkdirSync } from 'fs'

/**
 * Creates a directory if it doesn't exist.
 *
 * @example
 * ```javascript
 * createDirectoryIfNotExists('db/testnet') // creates `./db` and `./db/testnet`
 * ```
 * @param path
 */
export function createDirectoryIfNotExists(path: string): void {
  const chunks: string[] = path.split('/')

  chunks.reduce((searchPath: string, chunk: string) => {
    searchPath += '/'
    searchPath += chunk
    try {
      accessSync(`${process.cwd()}${searchPath}`)
    } catch (err) {
      mkdirSync(`${process.cwd()}${searchPath}`)
    }
    return searchPath
  }, '')
}
