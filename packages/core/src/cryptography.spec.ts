import { PRG as Rust_PRG } from './cryptography.js'
import { PRG as TS_PRG, PRP as TS_PRP } from '@hoprnet/hopr-utils'
import assert from 'assert'

describe('cryptographic correspondence tests', async function () {
  it('PRG correspondence', async function () {
    let key = new Uint8Array(16)
    let iv = new Uint8Array(12)

    {
      let rs_output = new Rust_PRG(key, iv).digest(5, 10)
      let ts_output = TS_PRG.createPRG({ key, iv }).digest(5, 10)
      assert.deepEqual(rs_output, ts_output)
    }

    {
      let rs_output = new Rust_PRG(key, iv).digest(0, 100)
      let ts_output = TS_PRG.createPRG({ key, iv }).digest(0, 100)
      assert.deepEqual(rs_output, ts_output)
    }

    {
      let rs_output = new Rust_PRG(key, iv).digest(10, 100)
      let ts_output = TS_PRG.createPRG({ key, iv }).digest(10, 100)
      assert.deepEqual(rs_output, ts_output)
    }

    {
      let rs_output = new Rust_PRG(key, iv).digest(16, 22)
      let ts_output = TS_PRG.createPRG({ key, iv }).digest(16, 22)
      assert.deepEqual(rs_output, ts_output)
    }
  })

  it('PRP correspondence', async function () {
    let key = new Uint8Array(16)
    let iv = new Uint8Array(12)
    let pt = new Uint8Array(400)

    let ts_prp = TS_PRP.createPRP({ key, iv })
    ts_prp.permutate(pt)
  })
})
