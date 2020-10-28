//import { RelayContext } from './relayContext'

describe('test overwritable connection', function () {
  /*
  let iteration = 0
  function getGenerator(): AsyncGenerator<Uint8Array> {
    return (async function* () {
      let i = 0
      for (; i < 23; i++) {
        yield new TextEncoder().encode(`iteration ${iteration} - msg no. ${i}`)
        await new Promise((resolve) => setTimeout(resolve, 12))
      }

      return `iteration ${iteration} - msg no. ${i + 1}`
    })()
  }
*/

  it('should create a connection and overwrite it', async function () {
    /*
    const ctx = new RelayContext(getGenerator())

    let i = setInterval(() => {
      ctx.update(getGenerator())
      iteration++
    }, 123)

    for await (const msg of ctx.source) {
      console.log(new TextDecoder().decode(msg))
    }

    clearInterval(i)
*/
  })
})
