import fs from 'fs'

// This file will contain the Runtime Environment Abstraction Layer (REAL) for Node.js or browser

class XOrError<X> {
  private d: X
  private e: any

  public get data() {
    return this.d
  }

  public set data(val) {
    this.d = val
    this.e = undefined
  }

  public get error() {
    return this.e
  }

  public set error(val) {
    this.e = val
    this.d = undefined
  }

  public hasError(): boolean {
    return this.e != undefined
  }
}

export class DataOrError extends XOrError<Uint8Array> { }


/**
 * Wrapper for reading file via WASM
 * @param file File path
 */
export function read_file(file: string): DataOrError {
  let ret = new DataOrError()
  try {
    ret.data = fs.readFileSync(file)
  } catch (e) {
    ret.error = e
  }

  return ret
}
