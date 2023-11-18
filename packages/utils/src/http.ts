import axios, { AxiosError } from 'axios'

export class HttpError {
  constructor(public msg: string, public httpStatus: number) { }
}

export async function post(url: string, json_data: string): Promise<string> {
  try {
    let response = await axios.post(url, json_data, {
      timeout: 30_000,
      maxRedirects: 3
    })

    return response.data.toString()
  }
  catch (err) {
    if (err instanceof AxiosError) {
      let error = err as AxiosError;
      throw new HttpError(error.message, error.status ?? -1)
    } else {
      throw new HttpError(err.toString(), -1)
    }
  }
}
