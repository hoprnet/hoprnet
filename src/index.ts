import { API_URL, BOT_NAME } from './env'
import { getHoprAddress  } from './utils'


const start = async () => {
  console.log(`Connecting to ${API_URL}`)
  const hoprAddress = await getHoprAddress()
  console.log(`My HOPR address is ${hoprAddress}`)

  switch(BOT_NAME) {
    case 'randobot': {
      const randobot = await import("./randobot");
      randobot.default(hoprAddress);
      break;
    }
    case 'bouncerbot': {
      const bouncerbot = await import("./bouncerbot");
      bouncerbot.default(hoprAddress);
      break;
    } 
  }
}

start().catch((err) => {
  console.error('Fatal Error:', err)
  process.exit();
})
