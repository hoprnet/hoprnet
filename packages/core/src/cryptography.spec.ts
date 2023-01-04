import { PRG as Rust_PRG, PRP as Rust_PRP } from './cryptography.js'
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

    let ts_prp = TS_PRP.createPRP({key, iv})
    let rs_prp = new Rust_PRP(key, iv)

    {
      let pt = new Uint8Array(400)
      let ct_1 = rs_prp.forward(pt)
      let ct_2 = ts_prp.permutate(pt)

      assert.deepEqual(ct_1, ct_2)
    }

    {
      let pt_1 = new Uint8Array(400)
      let ct = ts_prp.permutate(pt_1)
      let pt_2 = rs_prp.inverse(ct)

      assert.deepEqual(pt_1, pt_2)
    }

    {
      let pt_1 = new Uint8Array(400)
      let ct = rs_prp.forward(pt_1)
      let pt_2 = ts_prp.inverse(ct)

      assert.deepEqual(pt_1, pt_2)
    }
  })
})
