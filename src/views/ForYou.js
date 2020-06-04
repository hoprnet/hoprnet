import React from 'react'
import Blog from '../components/sections/Blog'
import ForYou from '../components/sections/ForYou'

class HOPR extends React.Component {
  render() {
    return (
      <React.Fragment>
        <ForYou id="for_you" />
        <Blog id="blog" hasBgColor invertColor />
      </React.Fragment>
    )
  }
}

export default HOPR
