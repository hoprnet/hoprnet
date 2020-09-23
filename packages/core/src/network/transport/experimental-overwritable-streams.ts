import Defer from 'p-defer'

let iteration = 0
function getGenerator() {
  return (async function* () {
    let i = 0
    for (; i < 23; i++) {
      yield `iteration ${iteration} - msg no. ${i}`
      await new Promise((resolve) => setTimeout(resolve, 123))
    }

    return `iteration ${iteration} - msg no. ${i + 1}`
  })()
}

async function main() {
  let defer = Defer<AsyncGenerator<string, string>>()
  let gen1 = getGenerator()
  const it4 = (async function* () {
    let msgReceived = false
    let streamReceived = false

    defer.promise.then(() => {
      streamReceived = true
    })

    let itDone = false

    let msg: Promise<IteratorResult<string, string>>
    while (true) {
      msg = gen1.next()

      await Promise.race([
        msg.then(({ done }) => {
          if (done) {
            itDone = true
          }

          msgReceived = true
        }),
        defer.promise,
      ])

      if (itDone || streamReceived) {
        console.log(`waiting for resolve`)
        gen1 = await defer.promise

        defer = Defer()

        streamReceived = false

        defer.promise.then(() => {
          console.log(`stream resolved`)
          streamReceived = true
        })

        itDone = false
        continue
      }

      if (msgReceived) {
        yield await msg
        msgReceived = false
      }
    }
  })()

  setInterval(() => {
    defer.resolve(getGenerator())
    defer = Defer()
  }, 1234)

  for await (const msg of it4) {
    console.log(msg)
  }
}

main()
