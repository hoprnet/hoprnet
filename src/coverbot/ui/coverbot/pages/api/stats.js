export function get(){
  let initialState = {
      address: '',
      available: 0,
      locked: 0,
      claimed: 0,
      connected: [
        /*
        {id: '0x12345', locked: 12, claimed: 0},
        */
      ],
      refreshed: new Date().toISOString()
    }
  return initialState 
}

export default (req, res) => {
  res.statusCode = 200
  res.json(get())
}
