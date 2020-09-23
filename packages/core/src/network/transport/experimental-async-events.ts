import { EventEmitter } from 'events'
import Defer, { DeferredPromise } from 'p-defer'

const emitter = new EventEmitter()

const it = (async function* foo() {
  const msgs: Uint8Array[] = []
  let done = false
  let defer = Defer<DeferredPromise<any>>()

  emitter.on('data', (data: Uint8Array) => {
    msgs.push(data)
    console.log(`msgs when pushing`, msgs.length)
    defer.resolve(Defer())
  })
  emitter.on('end', () => {
    console.log('done')
    done = true
  })

  while (true) {
    while (msgs.length > 0) {
      yield msgs.shift()
    }

    if (done) {
      console.log(`msgs before return`, msgs)
      return
    }

    defer = await defer.promise
  }
})()

async function main() {
  let i = 0

  const interval = setInterval(() => {
    if (++i == 15) {
      emitter.emit('end', new TextEncoder().encode('end'))

      clearInterval(interval)
    } else {
      emitter.emit('data', new TextEncoder().encode('data'))
    }
  }, 150)

  for await (const msg of it) {
    console.log(new TextDecoder().decode(msg))
    await new Promise((resolve) => setTimeout(resolve, 400))
  }
}

main()
