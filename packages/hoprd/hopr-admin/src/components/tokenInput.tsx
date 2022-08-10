import React from 'react'
import Cookies from 'js-cookie'

export default class TokenInput extends React.Component {
  constructor(props) {
    super(props)
    this.handleKeyPress = this.handleKeyPress.bind(this)
  }

  handleKeyPress(e) {
    if (e.key == 'Enter') {
      var text = e.target.value
      Cookies.set('X-Auth-Token', text)
      // this.props.handleTokenSet()
    }
  }

  render() {
    const tokenCookie = Cookies.get('X-Auth-Token')

    return tokenCookie === undefined ? (
      <div className="send">
        <input
          className="token"
          onKeyPress={this.handleKeyPress}
          id="token"
          type="password"
          placeholder="security token"
        />
      </div>
    ) : null
  }
}
