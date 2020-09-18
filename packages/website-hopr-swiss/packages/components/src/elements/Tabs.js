import React, { createContext } from 'react'
import PropTypes from 'prop-types'
import classNames from 'classnames'

const propTypes = {
  active: PropTypes.string,
}

const defaultProps = {
  active: null,
}

const context = createContext({})

const { Provider, Consumer } = context

const TabList = ({ className, ...props }) => {
  const classes = classNames('tab-list list-reset mb-0', className)

  return <ul {...props} className={classes} />
}

const Tab = ({ tabId, className, ...props }) => {
  return (
    <Consumer>
      {({ activeId, changeTab }) => (
        <li
          {...props}
          className={classNames('tab', activeId === tabId && 'is-active', className)}
          role="tab"
          aria-controls={tabId}
          onClick={() => changeTab(tabId)}
        />
      )}
    </Consumer>
  )
}

const TabPanel = ({ id, className, ...props }) => {
  return (
    <Consumer>
      {({ activeId }) => (
        <div
          {...props}
          id={id}
          className={classNames('tab-panel', activeId === id && 'is-active', className)}
          role="tabpanel"
        />
      )}
    </Consumer>
  )
}

class Tabs extends React.Component {
  state = {
    activeId: this.props.active,
  }

  changeTab = tabId => {
    this.setState({
      activeId: tabId,
    })
  }

  render() {
    const { className, active, ...props } = this.props

    const classes = classNames('tabs', className)

    return (
      <Provider
        value={{
          activeId: this.state.activeId,
          changeTab: this.changeTab,
        }}
      >
        <div {...props} className={classes}>
          {this.props.children}
        </div>
      </Provider>
    )
  }
}

Tabs.propTypes = propTypes
Tabs.defaultProps = defaultProps

export default Tabs
export { TabList, Tab, TabPanel }
