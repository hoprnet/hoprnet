import { debug, HoprDB } from '@hoprnet/hopr-utils'
import sinon from 'sinon'
import { NAMESPACE } from './constants'

const dbLogger = debug(`${NAMESPACE}:db`)

let db: HoprDB
db = sinon.createStubInstance(HoprDB)
db.close = () => {
  dbLogger('Closing database')
  return Promise.resolve()
}

const dbMock = db
export { dbMock }
