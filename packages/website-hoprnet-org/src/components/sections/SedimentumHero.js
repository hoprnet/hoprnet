import React from 'react'
import PropTypes from 'prop-types'
import GenericSection from './GenericSection'
import SedimentumTestimonial from './SedimentumTestimonial'
import Image from '../elements/Image'
import { SectionProps } from '../utils/SectionProps'

const propTypes = {
  children: PropTypes.node,
  ...SectionProps.types,
}

const defaultProps = {
  children: null,
  ...SectionProps.defaults,
}

const Sedimentum = props => {
  const oddSections = {
    hasBgColor: props.hasBgColor,
    invertColor: props.invertColor,
  }

  const evenSections = {
    hasBgColor: !oddSections.hasBgColor,
    invertColor: !oddSections.invertColor,
  }

  return (
    <div className="sedimentum">
      <GenericSection {...oddSections}>
        <div className="center-content">
          <div
            className="container-sm"
            style={{
              display: 'flex',
              justifyContent: 'center',
              alignItems: 'center',
            }}
          >
            <div className="mb-32 hero-figure reveal-from-top" data-reveal-delay="200">
              <Image
                className="has-shadow"
                src={require('@hoprnet/assets/images/partners/sedimentum_colored_transparent.png')}
                alt="Hero"
                width={896 * 0.5}
                height={504 * 0.5}
                style={{
                  borderRadius: '15px',
                  padding: '50px',
                }}
              />
            </div>
          </div>
          <div className="center-content" id="video">
            <div className="container-sm">
              <h2 className="section-header mt-0 mb-0 reveal-from-top" data-reveal-delay="150">
                SEDIMENTUM MAKES LIVING SAFE
              </h2>
              <div className="container reveal-from-top mb-24" data-reveal-delay="300">
                Sedimentum develops the first contactless solution for fall and emergency detection in healthcare
                without compromising privacy.
              </div>
              <iframe
                title="Sedimentum's safe-living device - how it works"
                width="800"
                height="450"
                src="https://www.youtube-nocookie.com/embed/S0_pH615RUY"
                frameBorder="0"
                allow="accelerometer; autoplay; encrypted-media; gyroscope; picture-in-picture"
                allowFullScreen
              />
            </div>
          </div>
          <SedimentumTestimonial />
        </div>
      </GenericSection>
      <GenericSection id="how" {...evenSections}>
        <div className="center-content">
          <div className="container-sm">
            <h2 className="section-header mt-0 mb-0 reveal-from-top" data-reveal-delay="150">
              How Sedimentum benefits from metadata privacy with HOPR
            </h2>
            <div className="container reveal-from-top mb-24" data-reveal-delay="300">
              <h4>Who</h4>
              <span>
                Sedimentum - Privacy-preserving AI in healthcare Sedimentum is a Swiss-based medtech startup which
                develops a novel technological protection mechanism to ensure the physical safety of unattended
                individuals (for example patients and elderly people) without compromising their privacy. Their
                technological solution aims to support and furthermore relieve the nursing staff in psychiatric
                hospitals and elderly care institutions. The medical staff gets informed in real-time about health
                related irregularities of their patients, for example triggered by falling accidentally. Thereby they
                are able to take the necessary actions to protect their patients.
                <br />
                <br />
                HOPR - privacy for the web of today and tomorrow HOPR is a privacy startup that provides metadata
                protection. Any web or decentralized application that sends or receives data can leverage the HOPR
                network to enable network-level privacy. HOPR is a decentralized and privacy-preserving messaging
                protocol with incentives for network participants who relay traffic of other peers. The HOPR team
                believes in and is working towards a decentralized web3 which is providing privacy by default and is
                more resilient against rent-seeking and data-hungry intermediaries than today’s web.
              </span>
            </div>
            <div className="container reveal-from-top mb-24" data-reveal-delay="300">
              <h4>Why</h4>
              <span>
                The continuous care of vulnerable persons, such as psychiatric patients or elderly people, is extremely
                labor intensive and therefore costly. It is not possible to guarantee the physical safety of patients
                with care needs at all times. This situation is a major challenge in psychiatric hospitals and elderly
                care institutions equally. The necessary human resources are often not sufficiently available to provide
                care services seamlessly. In addition, there are currently no effective technical aids available that
                could mitigate the problem described. Furthermore the requirements to successfully fulfill data privacy
                laws and regulatory aspects are tremendously complex. The healthcare sector and by its nature highly
                sensitive patient data requires privacy-preserving techniques at its best.
              </span>
            </div>
            <div className="container reveal-from-top mb-24" data-reveal-delay="300">
              <h4>How</h4>
              <span>
                The technological solution from Sedimentum can automatically detect severe incidents like falls or
                respiratory arrests of patients in a purely contactless manner. This is done by the combination of a
                wide variety of sensors, algorithms from the field of artificial intelligence (AI-Software) and a unique
                data anonymization procedure. In each patient room, a sensor device is mounted on the ceiling. The
                sensor device is able to measure different kind of health related data in a contactless fashion. It then
                transmits them fully anonymized to the high performance processing facilities hosted at Sedimentum to
                run the sophisticated AI algorithms. This AI-software learns, based on the transmitted sensor data, a
                physical reference state for each individual patient. Critical incidents, such as falls, can be spotted
                by the AI-software automatically through detecting irregularities in the sensor data. This triggers an
                alert to the nursing staff, so that they are able to perform the necessary actions.
                <br />
                <br />
                So far so good but the pivotal question is, how it is possible to collect data, analyze them and send an
                alert to the nursing staff in a truly privacy-preserving manner?
              </span>
            </div>
            <div className="container reveal-from-top mb-24" data-reveal-delay="300">
              <h4>Privacy requirements</h4>
              <span>
                Privacy always comes at some cost. In general it requires an increase in system complexity and one has
                to deal with some degree of information loss. It is recommendable to understand the specific privacy
                requirements in the specific field as well as possible. Sedimentum has identified the following privacy
                requirements in their use case:
                <br />
                <br />
                <ol>
                  <li>We must not know who is sending data (metadata protection)</li>
                  <li>We must not know who is the recipient of an alert (metadata protection)</li>
                  <li>
                    We must not receive any additional personal information besides the raw sensor values (privacy by
                    design)
                  </li>
                  <li>
                    It shall not be possible to link an alert back to its data source or vice versa (data
                    re-identification protection)
                  </li>
                </ol>
                <br />
                To be resilient against data re-identification attacks, Sedimentum had to develop a novel cryptographic
                protocol which makes it possible to route an alert to its correct recipient without knowing anything
                about the recipient itself. Sounds strange to you? Welcome to the counterintuitive feeling, almost
                magically world of modern cryptography.
                <br />
                <br />
                The mentioned cryptographic protocol relies heavily on an anonymous communication channels between the
                participating parties. I.e., it shall not be possible to observe who is communicating with whom - strong
                metadata protection is needed.
              </span>
            </div>
            <div className="container reveal-from-top mb-24" data-reveal-delay="300">
              <h4>How HOPR brings metadata privacy to Sedimentum</h4>
              <span>
                The data exchange between Sedimentum’s sensors in the patients’ rooms, all inter-party communication in
                the previous mentioned cryptographic protocol, the data processing facility and the personnel which
                receive alerts is based on HOPR. As a result, a third party such as internet service providers, cloud
                operators of the processing facility or even Sedimentum themselves can not obtain insights into events
                that can be linked to a patient. This unlinkability provides strong metadata privacy guarantees beyond
                simply omitting patient-specific metadata.
                <br />
                <br />
                <Image
                  className="has-shadow"
                  src={require('@hoprnet/assets/images/partners/sedimentum_diagram_1.png')}
                  alt="Sedimentum Diagram 1"
                  width={528}
                  height={396}
                />
                <br />
                <i>
                  Visualization of the data paths which are routed through the HOPR network for gaining metadata privacy
                </i>
                <br />
                <br />
                HOPR achieves this network level privacy by relaying data packets via intermediate relay hops. These
                HOPR relay nodes route traffic through a decentralized P2P network to obfuscate origin and destination
                of a data packet. In this way, the data is not just end-to-end encrypted but the origin and the
                destination of a data packet stays protected as well. In contrast, to e.g. a VPN operator, a HOPR node
                operator does not know if they receive traffic from the sender or another intermediate node. They also
                do not learn if they send data to the recipient or another intermediate relay node.
              </span>
            </div>
          </div>
        </div>
      </GenericSection>
      <GenericSection id="contact" {...oddSections}>
        <div className="center-content">
          <div className="contact-container container-sm">
            <div className="ta-l ">
              <Image
                className="has-shadow"
                src={require('@hoprnet/assets/images/partners/sedimentum_colored.jpg')}
                alt="Hero"
                width="350px"
              />
              <br />
              Contact Sedimentum AG
              <br />
              <br />
              Sumpfstrasse 26, 6302 Zug
              <br />
              Email:{' '}
              <a href="mailto:info@sedimentum.com" target="_blank" rel="noopener noreferrer" className="underline">
                info@sedimentum.com
              </a>
            </div>
            <div className="ta-l">
              <Image
                className="has-shadow"
                src={require('../assets/images/logo_gradient.png')}
                alt="Hero"
                width="200px"
              />
              <br />
              Contact HOPR
              <br />
              <br />
              Hardturmstrasse 161, 8005 Zürich
              <br />
              Email:{' '}
              <a href="mailto:partner@hoprnet.org" target="_blank" rel="noopener noreferrer" className="underline">
                partner@hoprnet.org
              </a>
            </div>
          </div>
        </div>
      </GenericSection>
    </div>
  )
}

Sedimentum.propTypes = propTypes
Sedimentum.defaultProps = defaultProps

export default Sedimentum
