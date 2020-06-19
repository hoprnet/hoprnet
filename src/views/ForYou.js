import React from 'react'
import Blog from '../components/sections/Blog'
import ForYou from '../components/sections/ForYou'
import Videos from '../components/sections/Videos'

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
