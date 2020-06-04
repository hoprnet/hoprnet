import React from 'react'
import PropTypes from 'prop-types'
import GenericSection from './GenericSection'
import Image from '../elements/Image'
import { SectionProps } from '../../utils/SectionProps'

const propTypes = {
  children: PropTypes.node,
  ...SectionProps.types,
}

const defaultProps = {
  children: null,
  ...SectionProps.defaults,
}

const youtubeIds = ['mcnezYJXuXw', 'wH48dy6PjVg', 'YN8BEF1JIQ0', 'lHQBiZmCLBY', 'kZiCoR1DYSg']

const OpenSource = props => {
  return (
    <>
      <GenericSection {...props}>
        <div className="center-content">
          <div className="container-ms">
            <h2 className="section-header mt-0 mb-0 reveal-from-top" data-reveal-delay="150">
              Open Source Support:
            </h2>
            <div className="open-source pt-32 reveal-from-top" data-reveal-delay="300">
              At HOPR we embrace and live the ethos of free and open source software. Especially when making claims
              about privacy, it is important to us, that you can check and challenge every bit of our work.
              <br />
              <br />
              But we want to take it further: Our vision of the web3 is an open and collaborative ecosystem. Therefore,
              we walk the talk and contribute to a range of open source projects beyond our primary project, the HOPR
              protocol:
              <br />
              <br />
              <ul>
                <li>
                  <a href="https://libp2p.io/">Libp2p</a> is a fundamental building block for a range of decentralized
                  projects such as Ethereum, Filecoin, IPFS, Polkadot and many more, we solved{' '}
                  <a href="https://github.com/libp2p/js-peer-id/pull/115">several</a>{' '}
                  <a href="https://github.com/libp2p/js-libp2p/pull/608">issues</a>{' '}
                  <a href="https://github.com/libp2p/js-peer-id/pull/116">and</a>{' '}
                  <a href="https://github.com/libp2p/js-libp2p/pull/330">contributed</a>{' '}
                  <a href="https://github.com/libp2p/js-peer-info/pull/91">various</a>{' '}
                  <a href="https://github.com/libp2p/js-peer-info/pull/89">improvements</a>
                </li>
                <li>
                  <a href="https://multiformats.io/">Multiformats</a> is a general-purpose value description format used
                  by a range of decentralized networks to which we contributed some{' '}
                  <a href="https://github.com/multiformats/js-multiaddr/pull/112">fixes and test</a>.
                </li>

                <li>
                  <a href="https://github.com/dignifiedquire/pull-length-prefixed">Pull-length-prefixed</a> is a data
                  transmission mechanism for decentralized applications where we{' '}
                  <a href="https://github.com/dignifiedquire/pull-length-prefixed/pull/20">resolved a critical issue</a>
                  .
                </li>

                <li>
                  <a href="http://definitelytyped.org/">DefinitelyTyped</a> is a repository for TypeScript definitions
                  used in over 3 million projects where we{' '}
                  <a href="https://github.com/DefinitelyTyped/DefinitelyTyped/pull/42559">
                    identified and resolved an implementation issue
                  </a>
                  .
                </li>
              </ul>
            </div>
          </div>
        </div>
      </GenericSection>
      <GenericSection {...props} hasBgColor invertColor id={undefined}>
        <div className="center-content">
          <div className="container-ms">
            <h2 className="section-header mt-0 mb-0 reveal-from-top" data-reveal-delay="150">
              Videos:
            </h2>
            <div className="pt-32 reveal-from-top" data-reveal-delay="300">
              {youtubeIds.map(id => (
                <iframe
                  title={id}
                  width="400"
                  height="225"
                  src={`https://www.youtube-nocookie.com/embed/${id}`}
                  frameborder="0"
                  allow="accelerometer; autoplay; encrypted-media; gyroscope; picture-in-picture"
                  allowfullscreen
                />
              ))}
            </div>
          </div>
        </div>
      </GenericSection>
      <GenericSection {...props} id={undefined}>
        <div className="center-content">
          <div className="container-ms">
            <h2 className="section-header mt-0 mb-0 reveal-from-top" data-reveal-delay="150">
              Documentation:
            </h2>
            <div className="pt-32 reveal-from-top" data-reveal-delay="300">
              <a href="http://docs.hoprnet.io/">Hosted at gitbook</a>
            </div>
          </div>
        </div>
      </GenericSection>
      <GenericSection {...props} hasBgColor invertColor id={undefined}>
        <div className="center-content">
          <div className="container-ms">
            <h2 className="section-header mt-0 mb-0 reveal-from-top" data-reveal-delay="150">
              Gitcoin Bounties:
            </h2>
            <div className="pt-32 reveal-from-top" data-reveal-delay="300">
              <a href="https://gitcoin.co/hoprnet">(coming soon mid July 2020)</a>
            </div>
          </div>
        </div>
      </GenericSection>
    </>
  )
}

OpenSource.propTypes = propTypes
OpenSource.defaultProps = defaultProps

export default OpenSource
