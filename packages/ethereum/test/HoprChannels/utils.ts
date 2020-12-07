import type { AsyncReturnType } from 'type-fest'
import type { HoprChannelsInstance } from '../../types'

export const formatAccount = (res: AsyncReturnType<HoprChannelsInstance['accounts']>) => ({
  secret: res[0],
  counter: res[1]
})

export const formatChannel = (res: AsyncReturnType<HoprChannelsInstance['channels']>) => ({
  deposit: res[0],
  partyABalance: res[1],
  closureTime: res[2],
  status: res[3],
  closureByPartyA: res[4]
})
