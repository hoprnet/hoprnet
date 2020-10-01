import React from 'react'
import classNames from 'classnames'
import { SectionProps } from '../utils/SectionProps'
import { Link } from 'react-router-dom'
import SectionHeader from './partials/SectionHeader'
import Input from '../elements/Input'
import Button from '../elements/Button'

const propTypes = {
  ...SectionProps.types,
}

const defaultProps = {
  ...SectionProps.defaults,
}

class SignupForm extends React.Component {
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
      title: 'Welcome. We exist to make entrepreneurship easier.',
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
                        <Input label="Full name" placeholder="Full name" labelHidden required />
                      </div>
                      <div className="mb-12">
                        <Input type="email" label="Email" placeholder="Email" labelHidden required />
                      </div>
                      <div className="mb-12">
                        <Input type="password" label="Password" placeholder="Password" labelHidden required />
                      </div>
                      <div className="mt-24 mb-32">
                        <Button color="primary" wide>
                          Sign up
                        </Button>
                      </div>
                    </fieldset>
                  </form>
                  <div className="signin-bottom has-top-divider">
                    <div className="pt-32 text-xs center-content text-color-low">
                      Already have an account?{' '}
                      <Link to="/login/" className="func-link">
                        Login
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

SignupForm.propTypes = propTypes
SignupForm.defaultProps = defaultProps

export default SignupForm
