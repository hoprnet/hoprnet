import fs from 'fs'

// This file will contain the Runtime Environment Abstraction Layer (REAL) for Node.js or browser

export class DataOrError {
  private d: Uint8Array
  private e: any

  get data() {
    return this.d
  }

  set data(val) {
    this.d = val
    this.e = undefined
  }

  get error() {
    return this.e
  }

  set error(val) {
    this.e = val
    this.d = undefined
  }
}

/**
 * Wrapper for reading file via WASM
 * @param file File path
 */
export function read_file_to_string(file: string): DataOrError {
  let ret = new DataOrError()
  try {
    ret.data = fs.readFileSync(file)
  } catch (e) {
    ret.error = e
  }

  return ret
}
