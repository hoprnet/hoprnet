import fs from 'fs'

// This file will contain the Runtime Environment Abstraction Layer (REAL) for Node.js or browser

/**
 * Simple base class for working-around the impossibility of Rust wasm_bindgen to handle
 * Result<X, JsValue> return type for bound JS functions which can throw an exception.
 *
 * Create a subclass of this base class to return value or error when exception is being caught
 * in the JS function. See `DataOrError` and `read_file` function in this module as an example/
 */
abstract class XOrError<X> {
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

/**
 * Class wrapping Uint8Array or error.
 */
export class DataOrError extends XOrError<Uint8Array> {}

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
