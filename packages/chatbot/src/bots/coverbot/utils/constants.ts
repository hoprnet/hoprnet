import { COVERBOT_VERIFICATION_CYCLE_IN_MS, HOPR_ENVIRONMENT, FIREBASE_DATABASE_URL } from '../../../utils/env'
import db from './db'

export const RELAY_VERIFICATION_CYCLE_IN_MS = COVERBOT_VERIFICATION_CYCLE_IN_MS * 10
export const RELAY_HOPR_REWARD = 1000000000000000 // 0.001 HOPR

export const scoreDbRef = db.ref(`/${HOPR_ENVIRONMENT}/score`)
export const stateDbRef = db.ref(`/${HOPR_ENVIRONMENT}/state`)
export const databaseTextRef = `${FIREBASE_DATABASE_URL} @ Schema ${HOPR_ENVIRONMENT}`

export enum ScoreRewards {
  verified = 100,
  relayed = 10,
}
