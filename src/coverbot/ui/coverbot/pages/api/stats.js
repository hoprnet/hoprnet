import fs from 'fs'

export function get(){
  try {
    let pth = process.env.STATS_FILE 
    let data = JSON.parse(fs.readFileSync(pth, 'utf8'))
    return data 
  } catch (e) {
    console.log(e)
    return {}
  }
}

export default (req, res) => {
  res.statusCode = 200
  res.json(get())
}
