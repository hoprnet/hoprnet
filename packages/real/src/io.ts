import fs from 'fs'

// This file will contain the Runtime Environment Abstraction Layer (REAL) for Node.js or browser

/**
 * Wrapper for reading file via WASM
 * @param file File path
 */
export function read_file(file: string): Uint8Array {
  return fs.readFileSync(file)
}

/**
 * Wrapper for reading file via WASM.
 * @param file File path
 * @param data Data to write to the file
 */
export function write_file(file: string, data: Uint8Array) {
  fs.writeFileSync(file, data)
}
