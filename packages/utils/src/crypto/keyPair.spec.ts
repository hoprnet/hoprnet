import { serializeKeyPair, deserializeKeyPair } from './keyPair'
import { privKeyToPeerId, stringToU8a, u8aEquals, u8aToHex } from '..'
import assert from 'assert'

describe('Identity', function () {
  const DUMMY_PASSWORD = 'hopr-unit-test-password'
  const WRONG_DUMMY_PASSWORD = 'hopr-unit-test-wrong-password'
  const DUMMY_PRIVATE_KEY = 'cd09f9293ffdd69be978032c533b6bcd02dfd5d937c987bedec3e28de07e0317'
  const TESTING_SALT = '0xbf73fe6f5a591c21e86fcce7a4f7c4925ed6e936dfab778ad907eed81dbcc56e'
  const TESTING_IV = '0x8532e739dc4c86d3c1ac9bd4be38a103'
  const TESTING_UUID = '0x2454ad016554ba1946a29e6a2d6beb22'
  const ENCODED_STRING =
    '0x7b226964223a2232343534616430312d363535342d346131392d383661322d396536613264366265623232222c2276657273696f6e223a332c2263727970746f223a7b22636970686572223a226165732d3132382d637472222c22636970686572706172616d73223a7b226976223a223835333265373339646334633836643363316163396264346265333861313033227d2c2263697068657274657874223a2232383239313634326638626336313832333238356264383435666433393066623339626465303037633365316365653464656130623761306630663836653130222c226b6466223a22736372797074222c226b6466706172616d73223a7b2273616c74223a2262663733666536663561353931633231653836666363653761346637633439323565643665393336646661623737386164393037656564383164626363353665222c226e223a312c22646b6c656e223a33322c2270223a312c2272223a387d2c226d6163223a2235626137396664376136626162343466353961393330656535626434653638643664616632663763616239643662326361613363396363653130313734313439227d7d'

  const TEST_PEERID = privKeyToPeerId(DUMMY_PRIVATE_KEY)

  describe('Private Key', () => {
    it('serializeKeyPair', async function () {
      const serialized = await serializeKeyPair(
        TEST_PEERID,
        DUMMY_PASSWORD,
        true,
        TESTING_IV,
        TESTING_SALT,
        TESTING_UUID
      )

      console.log(u8aToHex(serialized))
      assert(u8aEquals(serialized, stringToU8a(ENCODED_STRING)))
    })

    it('deserialize serialized key pair', async function () {
      const serialized = stringToU8a(ENCODED_STRING)
      const deserialized = await deserializeKeyPair(serialized, DUMMY_PASSWORD, true)
      assert(deserialized.success, `Deserialization must work`)

      assert(deserialized.identity.equals(TEST_PEERID), `Deserialized peerId must match original peerId`)
    })

    it('deserialize serialized key pair - bad examples', async function () {
      const serialized = stringToU8a(ENCODED_STRING)

      const deserialized = await deserializeKeyPair(serialized, WRONG_DUMMY_PASSWORD, true)
      assert(deserialized.success == false, `Deserialization must not work`)

      assert(deserialized.error === 'Invalid password', `Deserialization must fail with invalid password`)
    })
  })
})
