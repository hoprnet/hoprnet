import axios, { AxiosError } from 'axios'

export class HttpError {
  constructor(public msg: string, public httpStatus: number) {}
}

export type HttpConfig = {
  timeout_seconds: number
  max_redirects: number
}

export async function http_post(url: string, json_data: string, config: HttpConfig): Promise<string> {
  try {
    let response = await axios.post(url, json_data, {
      timeout: config.timeout_seconds * 1000,
      maxRedirects: config.max_redirects,
      headers: {
        'Accept-Encoding': 'gzip, compress, deflate',
        Accept: 'application/json',
        'Content-Type': 'application/json'
      }
    })

    return JSON.stringify(response.data)
  } catch (err) {
    if (err instanceof AxiosError) {
      let error = err as AxiosError
      throw new HttpError(error.message, error.status ?? -1)
    } else {
      throw new HttpError(err.toString(), -1)
    }
  }
}
