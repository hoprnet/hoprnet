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
export default function clearDirectory(path: string) {
    let files: string[] = []

    if (existsSync(path)) {
        files = readdirSync(path)
        files.forEach((file: string) => {
            const curPath = path + '/' + file
            if (lstatSync(curPath).isDirectory()) {
                // recurse
                clearDirectory(curPath)
            } else {
                // delete file
                unlinkSync(curPath)
            }
        })
        rmdirSync(path)
    }
}
