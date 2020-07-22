import * as grpc from 'grpc'
import { API_URL } from './env'

export const SetupClient = <T extends typeof grpc.Client>(Client: T): InstanceType<T> => {
  return (new Client(API_URL, grpc.credentials.createInsecure()) as unknown) as InstanceType<T>
}
