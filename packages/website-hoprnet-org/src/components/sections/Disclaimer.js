import React from 'react'
import PropTypes from 'prop-types'
import GenericSection from './GenericSection'
import Image from '../elements/Image'
import { SectionProps } from '../utils/SectionProps'

const isCompany = false

const propTypes = {
  children: PropTypes.node,
  ...SectionProps.types,
}

const defaultProps = {
  children: null,
  ...SectionProps.defaults,
}

const Disclaimer = props => {
  return (
    <GenericSection {...props}>
      <div className="center-content">
        <div className="container-sm">
          <h2 className="section-header mt-0 mb-0 reveal-from-top" data-reveal-delay="150">
            Disclaimer
          </h2>
          <div className="pt-32 ta-l reveal-from-top" data-reveal-delay="300">
            Imprint/Disclaimer
            <br />
            <br />
            {isCompany
              ? 'This website is operated by HOPR Services AG, Rebbergstrasse 33, 8706 Meilen, Switzerland.'
              : 'This website is operated by HOPR, c/o Froriep Legal AG, Bellerivestrasse 201, 8008 Zürich, Switzerland.'}
            <br />
            By using our website, you accept and agree to be bound and abide by the following terms and conditions. If
            you do not want to agree to these terms and conditions, you must not access or use our website.
            <br />
            The website, its content and any services or items obtained through the website are provided on an “as is”
            and “as available” basis, without any warranties of any kind, either express or implied.
            <br />
            <br />
            Neither we nor any person associated with us makes any warranty with respect to the completeness, security,
            reliability, quality, accuracy or availability of the website. Without limiting the foregoing, neither we
            nor anyone associated with us warrants that the website, its content or any services or items obtained
            through the website will be accurate, reliable, error-free or uninterrupted, that defects will be corrected,
            that our website or the server that makes it available are free of viruses or other harmful components or
            that the website or any services or items obtained through the website will otherwise meet your needs or
            expectations.
            <br />
            <br />
            To the fullest extent permitted by applicable law, in no event will we, our affiliates or our/their
            licensors, service providers, employees, agents, officers or directors be liable for damages of any kind,
            under any legal theory, arising out of or in connection with your use, or inability to use, the website, any
            websites linked to it, any content on the website or such other websites or any services or items obtained
            through the website or such other websites, including any direct, indirect, special, incidental, or
            consequential damages, including but not limited to, loss of revenue, loss of profits, loss of business or
            anticipated savings, loss of use, loss of goodwill, loss of data, even if foreseeable.
            <br />
            <br />
            We have no control over other websites. Links to other websites are neither a recommendation nor a guarantee
            or warranty, and do not mean that we agree with the contents of such websites. We reject responsibility for
            the contents of other websites. Your access and use is in your own risk.
          </div>
        </div>
      </div>
    </GenericSection>
  )
}

Disclaimer.propTypes = propTypes
Disclaimer.defaultProps = defaultProps

export default Disclaimer
