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
export declare function clearDirectory(path: string): void;
