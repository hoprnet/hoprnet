import fs from 'fs'

export function read_file(file: string): Uint8Array {
  return new Uint8Array(fs.readFileSync(file))
}
