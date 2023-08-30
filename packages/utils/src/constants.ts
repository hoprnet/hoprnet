import BN from 'bn.js'

// native balance (eth, xdai)
export const MIN_NATIVE_BALANCE = new BN('1000000000000000') // 0.001
export const SUGGESTED_NATIVE_BALANCE = MIN_NATIVE_BALANCE.muln(10) // 0.01
