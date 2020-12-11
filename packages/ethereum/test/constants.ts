import { createAccount } from './utils'

export const ACCOUNT_DEPLOYER_PRIVKEY = '0xf54bd518dd7e3e42710e9a96c92b1b244727df5a5afae34611089bee344d6bd4'
export const ACCOUNT_DEPLOYER = createAccount(ACCOUNT_DEPLOYER_PRIVKEY)

export const ACCOUNT_A_PRIVKEY = '0xf54bd518dd7e3e42710e9a96c92b1b244727df5a5afae34611089bee344d6bd4'
export const ACCOUNT_A = createAccount(ACCOUNT_A_PRIVKEY)

export const ACCOUNT_B_PRIVKEY = '0xf344315b0389d60ace0c8a5f36da6612d268019c2d88ff77cdb2b37f0ec7ddd5'
export const ACCOUNT_B = createAccount(ACCOUNT_B_PRIVKEY)
