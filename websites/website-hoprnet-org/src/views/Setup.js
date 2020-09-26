import React from 'react'
import { sections } from '../components'

const { Setup } = sections

class View extends React.Component {
  render() {
    return (
      <React.Fragment>
        <Setup hasBgColor invertColor />
      </React.Fragment>
    )
  }
}

export default View
