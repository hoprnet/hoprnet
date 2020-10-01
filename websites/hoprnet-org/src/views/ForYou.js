import React from 'react'
import { sections } from '../components'

const { Blog, ForYou, Videos } = sections

class HOPR extends React.Component {
  render() {
    return (
      <React.Fragment>
        <ForYou id="for_you" />
        <Videos id="videos" hasBgColor invertColor />
        {/* News Component */}
        <Blog id="blog" />
      </React.Fragment>
    )
  }
}

export default HOPR
