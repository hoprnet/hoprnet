import { RelayConnection } from './relayConnection'
import assert from 'assert'

interface PairType<T> {
  sink(source: AsyncIterable<T>): Promise<void>
  source: AsyncIterable<T>
}

const Pair: <T>() => PairType<T> = require('it-pair')

describe('test relay connection', function () {
  it('should initiate a relayConnection and exchange a demo message', async function () {
    // const AliceBob = Pair<Uint8Array>()
    // const BobAlice = Pair<Uint8Array>()
    // const a = new RelayConnection({
    //   stream: {
    //     sink: AliceBob.sink,
    //     source: BobAlice.source,
    //   },
    // })
    // const b = new RelayConnection({
    //   stream: {
    //     sink: BobAlice.sink,
    //     source: AliceBob.source,
    //   },
    // })
    // a.sink(
    //   (async function* () {
    //     let i = 0
    //     while (true) {
    //       yield new TextEncoder().encode(`message ${i++}`)
    //       await new Promise((resolve) => setTimeout(resolve, 100))
    //     }
    //   })()
    // )
    // setTimeout(() => {
    //   setImmediate(async () => {
    //     console.log(`close triggered`)
    //     b.close()
    //   })
    // }, 500)
    // for await (const msg of b.source) {
    //   console.log(new TextDecoder().decode(msg.slice()))
    // }
    // for await (const msg of a.source) {
    //   throw Error(`there should be no message`)
    // }
    // await new Promise((resolve) => setTimeout(resolve, 50))
    // console.log(`sender.resolved`)
    // console.log(`receiver.resolved`)
    // assert(b.destroyed && a.destroyed, `both parties must have marked the connection as destroyed`)
  })

  it('should initiate a relayConnection and exchange a demo message', async function () {
    const AliceBob = Pair<Uint8Array>()
    const BobAlice = Pair<Uint8Array>()

    const a = new RelayConnection({
      stream: {
        sink: AliceBob.sink,
        source: BobAlice.source,
      },
    })

    const b = new RelayConnection({
      stream: {
        sink: BobAlice.sink,
        source: AliceBob.source,
      },
    })

    a.sink(
      (async function* () {
        let i = 0
        while (true) {
          yield new TextEncoder().encode(`message ${i++}`)

          await new Promise((resolve) => setTimeout(resolve, 100))
        }
      })()
    )

    setTimeout(() => {
      setImmediate(async () => {
        console.log(`close triggered`)
        a.close()
      })
    }, 500)

    for await (const msg of b.source) {
      console.log(new TextDecoder().decode(msg.slice()))
    }

    // for await (const msg of a.source) {
    //   throw Error(`there should be no message`)
    // }

    await new Promise((resolve) => setTimeout(resolve, 50))
    console.log(`sender.resolved`)
    console.log(`receiver.resolved`)

    // @ts-ignore
    console.log(a.source.next())

    // @TODO
    // b.source.next()
    // b.sink.next() ? 
    
    assert(b.destroyed && a.destroyed, `both parties must have marked the connection as destroyed`)
  })
})
