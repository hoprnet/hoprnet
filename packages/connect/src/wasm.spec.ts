import { foo_bar, IStream } from '../lib/connect_relay.js'

describe('test wasm', function () {
  it('do stuff', async function () {
    let stream: IStream = {
      source: (async function* () {
        let i = 0
        while (true) {
          yield new Uint8Array([i++, i++, i++])
        }
      })(),
      sink: async function (source: AsyncIterable<Uint8Array>) {
        // console.log(source[Symbol.asyncIterator]().next())
        console.log(source)
        console.log(`js: sink called`)
        for await (const chunk of source) {
          console.log(`js: received chunk`, chunk)
        }
        console.log(`js: after iterator`)
      }
    }

    await foo_bar(stream)
  })
})
