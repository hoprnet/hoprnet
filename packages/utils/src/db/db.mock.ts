import { HoprDB } from '.'
import { debug } from '../process'

const dbLogger = debug(`hopr:mocks:db`)

let db: HoprDB
db = {} as unknown as HoprDB
db.close = () => {
  dbLogger('Closing database')
  return Promise.resolve()
}

const dbMock = db
export { dbMock }
