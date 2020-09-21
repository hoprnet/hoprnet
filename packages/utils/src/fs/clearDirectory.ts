import { existsSync, readdirSync, lstatSync, unlinkSync, rmdirSync } from 'fs'
/**
 * Deletes recursively (and synchronously) all files in a directory.
 *
 * @param path the path to the directory
 *
 * @example
 *
 * ```javascript
 * clearDirectory('./db')
 * // deletes all files and subdirectories in `./db`
 * ```
 */
export function clearDirectory(path: string): void {
  let files: string[]

  if (!existsSync(path)) {
    throw Error('Path does not exist.')
  }
  let toDelete: string[] = [path]

  let curPath: string
  while (toDelete.length > 0) {
    curPath = toDelete[toDelete.length - 1]
    files = readdirSync(curPath).map((file) => curPath + '/' + file)

    if (files.length > 0) {
      toDelete.push(...files)
    } else {
      toDelete.pop()
      if (lstatSync(curPath).isDirectory()) {
        rmdirSync(curPath)
      } else if (lstatSync(curPath).isFile()) {
        unlinkSync(curPath)
      } else {
        throw Error('not implemented')
      }
    }
  }
}
