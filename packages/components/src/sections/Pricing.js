import React from 'react'
import PropTypes from 'prop-types'
import classNames from 'classnames'
import { SectionTilesProps } from '../utils/SectionProps'
import SectionHeader from './partials/SectionHeader'
import Switch from '../elements/Switch'
import Button from '../elements/Button'

const propTypes = {
  ...SectionTilesProps.types,
  pricingSwitcher: PropTypes.bool,
  pricingSlider: PropTypes.bool,
}

const defaultProps = {
  ...SectionTilesProps.defaults,
  pricingSwitcher: false,
  pricingSlider: false,
}

class Pricing extends React.Component {
  state = {
    priceChangerValue: '1',
    priceInput: {
      0: '1,000',
      1: '1,250',
      2: '1,500',
      3: '2,000',
      4: '2,500',
      5: '3,500',
      6: '6,000',
      7: '15,000',
      8: '50,000',
    },
    priceOutput: {
      plan1: {
        0: ['', 'Free', ''],
        1: ['$', '13', '/m'],
        2: ['$', '17', '/m'],
        3: ['$', '21', '/m'],
        4: ['$', '25', '/m'],
        5: ['$', '42', '/m'],
        6: ['$', '58', '/m'],
        7: ['$', '117', '/m'],
        8: ['$', '208', '/m'],
      },
      plan2: {
        0: ['$', '13', '/m'],
        1: ['$', '17', '/m'],
        2: ['$', '21', '/m'],
        3: ['$', '25', '/m'],
        4: ['$', '42', '/m'],
        5: ['$', '58', '/m'],
        6: ['$', '117', '/m'],
        7: ['$', '208', '/m'],
        8: ['$', '299', '/m'],
      },
      plan3: {
        0: ['$', '17', '/m'],
        1: ['$', '21', '/m'],
        2: ['$', '25', '/m'],
        3: ['$', '42', '/m'],
        4: ['$', '58', '/m'],
        5: ['$', '117', '/m'],
        6: ['$', '208', '/m'],
        7: ['$', '299', '/m'],
        8: ['$', '499', '/m'],
      },
    },
  }

  slider = React.createRef()
  sliderValue = React.createRef()

  componentDidMount() {
    if (this.props.pricingSlider) {
      this.slider.current.setAttribute('min', 0)
      this.slider.current.setAttribute('max', Object.keys(this.state.priceInput).length - 1)
      this.thumbSize = parseInt(window.getComputedStyle(this.sliderValue.current).getPropertyValue('--thumb-size'), 10)
      this.handleSliderValuePosition(this.slider.current)
    }
  }

  handlePricingSwitch = e => {
    this.setState({ priceChangerValue: e.target.checked ? '1' : '0' })
  }

  handlePricingSlide = e => {
    this.setState({ priceChangerValue: e.target.value })
    this.handleSliderValuePosition(e.target)
  }

  handleSliderValuePosition = input => {
    const multiplier = input.value / input.max
    const thumbOffset = this.thumbSize * multiplier
    const priceInputOffset = (this.thumbSize - this.sliderValue.current.clientWidth) / 2
    this.sliderValue.current.style.left = input.clientWidth * multiplier - thumbOffset + priceInputOffset + 'px'
  }

  getPricingData = (values, set) => {
    return set !== undefined ? values[this.state.priceChangerValue][set] : values[this.state.priceChangerValue]
  }

  render() {
    const {
      className,
      topOuterDivider,
      bottomOuterDivider,
      topDivider,
      bottomDivider,
      hasBgColor,
      invertColor,
      pushLeft,
      pricingSwitcher,
      pricingSlider,
      ...props
    } = this.props

    const outerClasses = classNames(
      'pricing section cursor',
      topOuterDivider && 'has-top-divider',
      bottomOuterDivider && 'has-bottom-divider',
      hasBgColor && 'has-bg-color',
      invertColor && 'invert-color',
      className
    )

    const innerClasses = classNames(
      'pricing-inner section-inner',
      topDivider && 'has-top-divider',
      bottomDivider && 'has-bottom-divider'
    )

    const tilesClasses = classNames('tiles-wrap', pushLeft && 'push-left')

    const sectionHeader = {
      title: 'Simple, transparent pricing',
      paragraph:
        'Vitae aliquet nec ullamcorper sit amet risus nullam eget felis semper quis lectus nulla at volutpat diam ut venenatis tellus—in ornare.',
    }

    return (
      <section {...props} className={outerClasses}>
        <div className="container">
          <div className={innerClasses}>
            <SectionHeader data={sectionHeader} className="center-content" />
            {pricingSwitcher && (
              <div className="pricing-switcher center-content">
                <Switch
                  checked={this.state.priceChangerValue === '1' ? true : false}
                  onChange={this.handlePricingSwitch}
                  rightLabel="Billed Annually"
                >
                  Billed Monthly
                </Switch>
              </div>
            )}
            {pricingSlider && (
              <div className="pricing-slider center-content">
                <label className="form-slider">
                  <span className="mb-16">How many users do you have?</span>
                  <input
                    type="range"
                    ref={this.slider}
                    defaultValue={this.state.priceChangerValue}
                    onChange={this.handlePricingSlide}
                  />
                </label>
                <div ref={this.sliderValue} className="pricing-slider-value">
                  {this.getPricingData(this.state.priceInput)}
                </div>
              </div>
            )}
            <div className={tilesClasses}>
              <div className="tiles-item reveal-from-bottom">
                <div className="tiles-item-inner has-shadow">
                  <div className="pricing-item-content">
                    <div className="pricing-item-header pb-24 mb-24">
                      <div className="pricing-item-price mb-4">
                        <span className="pricing-item-price-currency h3">
                          {this.getPricingData(this.state.priceOutput.plan1, 0)}
                        </span>
                        <span className="pricing-item-price-amount h1">
                          {this.getPricingData(this.state.priceOutput.plan1, 1)}
                        </span>
                        <span className="pricing-item-price-after text-sm">
                          {this.getPricingData(this.state.priceOutput.plan1, 2)}
                        </span>
                      </div>
                      <div className="text-xs text-color-low">Lorem ipsum is a common text</div>
                    </div>
                    <div className="pricing-item-features mb-40">
                      <div className="pricing-item-features-title h6 text-xs text-color-high mb-24">
                        What’s included
                      </div>
                      <ul className="pricing-item-features-list list-reset text-xs mb-32">
                        <li className="is-checked">Excepteur sint occaecat velit</li>
                        <li className="is-checked">Excepteur sint occaecat velit</li>
                        <li className="is-checked">Excepteur sint occaecat velit</li>
                        <li>Excepteur sint occaecat velit</li>
                        <li>Excepteur sint occaecat velit</li>
                      </ul>
                    </div>
                  </div>
                  <div className="pricing-item-cta mb-8">
                    <Button tag="a" color="primary" wide href="http://cruip.com/">
                      Start free trial
                    </Button>
                  </div>
                </div>
              </div>

              <div className="tiles-item reveal-from-bottom" data-reveal-delay="200">
                <div className="tiles-item-inner has-shadow">
                  <div className="pricing-item-content">
                    <div className="pricing-item-header pb-24 mb-24">
                      <div className="pricing-item-price mb-4">
                        <span className="pricing-item-price-currency h3">
                          {this.getPricingData(this.state.priceOutput.plan2, 0)}
                        </span>
                        <span className="pricing-item-price-amount h1">
                          {this.getPricingData(this.state.priceOutput.plan2, 1)}
                        </span>
                        <span className="pricing-item-price-after text-sm">
                          {this.getPricingData(this.state.priceOutput.plan2, 2)}
                        </span>
                      </div>
                      <div className="text-xs text-color-low">Lorem ipsum is a common text</div>
                    </div>
                    <div className="pricing-item-features mb-40">
                      <div className="pricing-item-features-title h6 text-xs text-color-high mb-24">
                        What’s included
                      </div>
                      <ul className="pricing-item-features-list list-reset text-xs mb-32">
                        <li className="is-checked">Excepteur sint occaecat velit</li>
                        <li className="is-checked">Excepteur sint occaecat velit</li>
                        <li className="is-checked">Excepteur sint occaecat velit</li>
                        <li className="is-checked">Excepteur sint occaecat velit</li>
                        <li>Excepteur sint occaecat velit</li>
                      </ul>
                    </div>
                  </div>
                  <div className="pricing-item-cta mb-8">
                    <Button tag="a" color="primary" wide href="http://cruip.com/">
                      Start free trial
                    </Button>
                  </div>
                </div>
              </div>

              <div className="tiles-item reveal-from-bottom" data-reveal-delay="400">
                <div className="tiles-item-inner has-shadow">
                  <div className="pricing-item-content">
                    <div className="pricing-item-header pb-24 mb-24">
                      <div className="pricing-item-price mb-4">
                        <span className="pricing-item-price-currency h3">
                          {this.getPricingData(this.state.priceOutput.plan3, 0)}
                        </span>
                        <span className="pricing-item-price-amount h1">
                          {this.getPricingData(this.state.priceOutput.plan3, 1)}
                        </span>
                        <span className="pricing-item-price-after text-sm">
                          {this.getPricingData(this.state.priceOutput.plan3, 2)}
                        </span>
                      </div>
                      <div className="text-xs text-color-low">Lorem ipsum is a common text</div>
                    </div>
                    <div className="pricing-item-features mb-40">
                      <div className="pricing-item-features-title h6 text-xs text-color-high mb-24">
                        What’s included
                      </div>
                      <ul className="pricing-item-features-list list-reset text-xs mb-32">
                        <li className="is-checked">Excepteur sint occaecat velit</li>
                        <li className="is-checked">Excepteur sint occaecat velit</li>
                        <li className="is-checked">Excepteur sint occaecat velit</li>
                        <li className="is-checked">Excepteur sint occaecat velit</li>
                        <li className="is-checked">Excepteur sint occaecat velit</li>
                      </ul>
                    </div>
                  </div>
                  <div className="pricing-item-cta mb-8">
                    <Button tag="a" color="primary" wide href="http://cruip.com/">
                      Start free trial
                    </Button>
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

Pricing.propTypes = propTypes
Pricing.defaultProps = defaultProps

export default Pricing
