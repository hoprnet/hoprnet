import axios from 'axios'

export async function post(url: string, json_data: string): Promise<string> {
  let response = await axios.post(url, json_data, {
    timeout: 30_000,
    maxRedirects: 3
  })
  if (response.status >= 400) {
  throw new Error(`http return error code: ${response.status}`)
}

return response.data.toString()
}