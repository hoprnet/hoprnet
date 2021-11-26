import { debug, HoprDB } from '@hoprnet/hopr-utils'
import { NAMESPACE } from './constants'

const dbLogger = debug(`${NAMESPACE}:db`)

let db: HoprDB
db = {} as unknown as HoprDB
db.close = () => {
  dbLogger('Closing database')
  return Promise.resolve()
}

const dbMock = db
export { dbMock }
