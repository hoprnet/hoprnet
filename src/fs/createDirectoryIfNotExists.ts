import { existsSync, mkdirSync } from 'fs'

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
  if (path.endsWith('/')) {
    path = path.substring(0, path.length - 1)
  }
  const chunks: string[] = path.split('/')

  let searchPath = ''

  for (let i = 0; i < chunks.length; i++) {
    searchPath += chunks[i] + '/'

    if (!existsSync(searchPath)) {
      mkdirSync(searchPath)
    }
  }
}
