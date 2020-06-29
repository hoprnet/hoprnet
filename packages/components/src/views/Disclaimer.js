import React from 'react'
import { sections } from '@hoprnet/hopr-website.components'

const { Disclaimer } = sections

class Home extends React.Component {
  render() {
    return (
      <React.Fragment>
        <Disclaimer />
      </React.Fragment>
    )
  }
}

export default Home
