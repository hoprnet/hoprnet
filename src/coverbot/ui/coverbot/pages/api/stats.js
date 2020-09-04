import fs from 'fs'

export function get(){
  let data = JSON.parse(fs.readFileSync("../../stats.json", 'utf8'))
  return data 
}

export default (req, res) => {
  res.statusCode = 200
  res.json(get())
}
