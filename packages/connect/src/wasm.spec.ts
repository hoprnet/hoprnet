import { IStream, JsStreamingIterableOutput } from '../lib/connect_relay.js'

describe('test wasm', function () {
  it('do stuff', async function () {
    let stream: IStream = {
      source: (async function* () {
        let i = 0
        while (i < 11) {
          yield new Uint8Array([i++, i++, i++, i])
        }
      })(),
      sink: async function (source: AsyncIterable<Uint8Array>) {
        // console.log(source[Symbol.asyncIterator]().next())
        // console.log(source)
        console.log(`js: sink called`)
        for await (const chunk of source) {
          console.log(`js: received chunk`, chunk)
        }
        console.log(`js: after iterator`)
      }
    }

    const transformed = new JsStreamingIterableOutput(stream)

    // const foo = {
    //   sink: transformed.sink.bind(transformed),
    //   source: {
    //     [Symbol.asyncIterator]() {
    //       return transformed
    //     }
    //   }
    // }

    for await (const msg of transformed.source) {
      console.log(`inside for await`, msg)
    }

    await transformed.sink(
      (async function* () {
        yield Uint8Array.from([0x00, 0x01, 0x02])
        yield Uint8Array.from([0x03, 0x04, 0x05])
      })() as any
    )
  })
})
