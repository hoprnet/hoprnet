import fs from 'fs'

// This file will contain the Runtime Environment Abstraction Layer (REAL) for Node.js or browser

/**
 * Wrapper for reading file via WASM
 * @param file File path
 */
export function read_file(file: string): Uint8Array {
  return new Uint8Array(fs.readFileSync(file))
}
