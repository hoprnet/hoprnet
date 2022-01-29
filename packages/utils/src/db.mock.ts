import { debug, HoprDB } from '.'

const dbLogger = debug(`hopr:mocks:db`)

let db: HoprDB
db = {} as unknown as HoprDB
db.close = () => {
  dbLogger('Closing database')
  return Promise.resolve()
}

const dbMock = db
export { dbMock }
