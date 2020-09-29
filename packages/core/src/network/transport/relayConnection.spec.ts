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
    // // @ts-ignore
    // assert((await a.source.next()).done && (await b.source.next()).done, `Streams must have ended.`)
    // assert(b.destroyed && a.destroyed, `both parties must have marked the connection as destroyed`)
  })

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
    //     a.close()
    //   })
    // }, 500)
    // for await (const msg of b.source) {
    //   console.log(new TextDecoder().decode(msg.slice()))
    // }
    // for await (const msg of a.source) {
    //   throw Error(`there should be no message`)
    // }
    // await new Promise((resolve) => setTimeout(resolve, 50))
    // assert(
    //   // @ts-ignore
    //   (await Promise.all([a.source.next(), b.source.next()])).every(({ done }) => done),
    //   `Streams must have ended.`
    // )
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
          yield new TextEncoder().encode(`message from a ${i++}`)

          await new Promise((resolve) => setTimeout(resolve, 100))
        }
      })()
    )

    b.sink(
      (async function* () {
        let i = 0
        await new Promise((resolve) => setTimeout(resolve, 50))

        while (true) {
          yield new TextEncoder().encode(`message from b ${i++}`)

          await new Promise((resolve) => setTimeout(resolve, 100))
        }
      })()
    )

    setTimeout(() => {
      console.log(`close triggered`)
      a.close()
    }, 520)

    let msgAReceived = false
    let msgBReceived = false

    let aDone = false
    let bDone = false

    function aFunction({ done, value }) {
      msgAReceived = true
      if (done) {
        aDone = true
      }
      return { done, value }
    }

    function bFunction({ done, value }) {
      msgBReceived = true
      if (done) {
        bDone = true
      }
      return { done, value }
    }

    // @ts-ignore
    let msgA = a.source.next().then(aFunction)

    // @ts-ignore
    let msgB = b.source.next().then(bFunction)

    while (true) {
      console.log(`aDone`, aDone, `bDone`, bDone)
      if (!aDone && !bDone) {
        console.log(`before promise race`)
        await Promise.race([
          // prettier-ignore
          msgA,
          msgB,
        ])
        console.log(`after promise race`)
      } else if (aDone) {
        await msgB
      } else if (bDone) {
        await msgA
      } else {
        break
      }

      if (msgAReceived || bDone) {
        msgAReceived = false

        if (aDone && bDone) {
          break
        } else {
          console.log(new TextDecoder().decode((await msgA).value))
        }

        //@ts-ignore

        msgA = a.source.next().then(aFunction)
      }

      if (msgBReceived || aDone) {
        msgBReceived = false

        if (aDone && bDone) {
          break
        } else {
          console.log(new TextDecoder().decode((await msgB).value))
        }
        //@ts-ignore
        msgB = b.source.next().then(bFunction)
      }
    }

    // console.log(`here`)
    await new Promise((resolve) => setTimeout(resolve, 600))

    // @ts-ignore
    const results = await Promise.all([a.source.next(), b.source.next()])
    console.log(results)
    // assert.deepStrictEqual(
    //   // @ts-ignore
    //   results,
    //   undefined,
    //   `Streams must have ended.`
    // )
    assert(b.destroyed && a.destroyed, `both parties must have marked the connection as destroyed`)
  })
})
