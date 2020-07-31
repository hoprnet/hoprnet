import React from 'react'
import classNames from 'classnames'
import { SectionProps } from '../utils/SectionProps'
import { Link } from 'react-router-dom'
import SectionHeader from './partials/SectionHeader'
import Input from '../elements/Input'
import Button from '../elements/Button'
import Checkbox from '../elements/Checkbox'

const propTypes = {
  ...SectionProps.types,
}

const defaultProps = {
  ...SectionProps.defaults,
}

class LoginForm extends React.Component {
  render() {
    const {
      className,
      topOuterDivider,
      bottomOuterDivider,
      topDivider,
      bottomDivider,
      hasBgColor,
      invertColor,
      ...props
    } = this.props

    const outerClasses = classNames(
      'signin section cursor',
      topOuterDivider && 'has-top-divider',
      bottomOuterDivider && 'has-bottom-divider',
      hasBgColor && 'has-bg-color',
      invertColor && 'invert-color',
      className
    )

    const innerClasses = classNames(
      'signin-inner section-inner',
      topDivider && 'has-top-divider',
      bottomDivider && 'has-bottom-divider'
    )

    const sectionHeader = {
      title: 'Welcome back. We exist to make entrepreneurship easier.',
    }

    return (
      <section {...props} className={outerClasses}>
        <div className="container">
          <div className={innerClasses}>
            <SectionHeader tag="h1" data={sectionHeader} className="center-content" />
            <div className="tiles-wrap">
              <div className="tiles-item">
                <div className="tiles-item-inner">
                  <form>
                    <fieldset>
                      <div className="mb-12">
                        <Input type="email" label="Email" placeholder="Email" labelHidden required />
                      </div>
                      <div className="mb-12">
                        <Input type="password" label="Password" placeholder="Password" labelHidden required />
                      </div>
                      <div className="mt-24 mb-32">
                        <Button color="primary" wide>
                          Sign in
                        </Button>
                      </div>
                      <div className="signin-footer mb-32">
                        <Checkbox>Remember me</Checkbox>
                        <Link to="/recover-password/" className="func-link text-xs">
                          Forgot password?
                        </Link>
                      </div>
                    </fieldset>
                  </form>
                  <div className="signin-bottom has-top-divider">
                    <div className="pt-32 text-xs center-content text-color-low">
                      Don't you have an account?{' '}
                      <Link to="/signup/" className="func-link">
                        Sign up
                      </Link>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>
    )
  }
}

LoginForm.propTypes = propTypes
LoginForm.defaultProps = defaultProps

export default LoginForm
