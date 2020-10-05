import React from 'react'
import classNames from 'classnames'
import GenericSection from './GenericSection'
import SectionHeader from './partials/SectionHeader'
import { Accordion, AccordionItem } from '../elements'

const FAQ = (props = {}) => {
  const outerClasses = classNames(
    'cursor reveal-from-top',
    props.topOuterDivider && 'has-top-divider',
    props.bottomOuterDivider && 'has-bottom-divider',
    props.hasBgColor && 'has-bg-color',
    props.invertColor && 'invert-color',
    props.className
  )

  return (
    <GenericSection id={props.id} className={outerClasses} data-reveal-delay="300">
      <div className="container">
        <SectionHeader
          data={{
            title: 'FAQ',
          }}
          className="center-content"
        />
        <Accordion>
          {items.map(({ title, paragraph }) => {
            return <AccordionItem title={title}>{paragraph}</AccordionItem>
          })}
        </Accordion>
      </div>
    </GenericSection>
  )
}

const items = [
  {
    title: 'What is the difference between HOPR and VPNs? Why is HOPR better?',
    paragraph: (
      <>
        A VPN protects you from data harvesting website operators by hiding your identity behind their VPN server.
        However, by doing so they establish themselves as just another central data honeypot that regularly{' '}
        <a
          className="text-color-high underline"
          href="https://twitter.com/hoprnet/status/1284766948593721344"
          target="_blank"
          rel="noopener noreferrer"
        >
          leaks everyone’s entire browsing history and other connection metadata
        </a>{' '}
        of their users. HOPR does not have any central point of failure, trust or control and as such you do not even
        need to trust us because we do not operate the HOPR network. As long as one (out of typically three) relayers
        that send your data to the recipient is doing their job well, the connection data remain private. HOPR provides
        additional layers of data privacy protection by mixing packets along the route and making all packets look
        indistinguishable from one another. This is how HOPR establishes privacy in unprecedented ways.
      </>
    ),
  },
  {
    title: 'Why is the company named HOPR? And how is it pronounced?',
    paragraph: (
      <>
        The name HOPR (pronounced Hopper) came from the fact that metadata exchanged by our users will, essentially,
        “hop” from node to node, ensuring complete privacy. You can read more about the background of HOPR from our
        co-founder, Rik Krieger,{' '}
        <a
          className="text-color-high underline"
          href="https://medium.com/hoprnet/introducing-the-hopr-founders-how-hopr-came-to-be-1d60ec43a577"
          target="_blank"
          rel="noopener noreferrer"
        >
          here
        </a>
        .
      </>
    ),
  },
  {
    title: 'What stage of development is HOPR currently at?',
    paragraph: (
      <>
        HOPR is at the early stages of technical development, but we’re moving fast. We had our first public testnet
        just one week after our funding announcement. (You can read more about it{' '}
        <a
          className="text-color-high underline"
          href="https://medium.com/hoprnet/introducing-hopr-testnet-sessions-6acfca2c6b31"
          target="_blank"
          rel="noopener noreferrer"
        >
          here
        </a>
        .) While packet mixing is a core aspect of a mixnet, it is not the most critical development milestone. We’re
        focused on delivering working software to our community first, before enabling the more technical features which
        we’ve already developed in a way that doesn’t break the UX. We plan for a mainnet release in late 2020.
      </>
    ),
  },
  {
    title: 'Why hasn’t anyone done what HOPR’s done yet?',
    paragraph:
      'The HOPR protocol provides network-level and metadata privacy for every kind of data exchange, while introducing the first-ever open incentivized mixnet, where anyone can earn tokens for running nodes. HOPR’s incentivized mixnets is possible with the advent of blockchains with on-chain capabilities to run advanced layer-2 scaling solutions. And we have a capable team 😉.',
  },
  {
    title: 'How much has the project raised so far?',
    paragraph: (
      <>
        The project raised USD $1M from Binance Labs and other investors in our seed financing round. (Read more{' '}
        <a
          className="text-color-high underline"
          href="https://medium.com/hoprnet/binance-labs-leads-strategic-1-million-investment-round-in-hopr-113a22458406"
          target="_blank"
          rel="noopener noreferrer"
        >
          here
        </a>
        .) The team feels honored to be Binance’s first investment of the year – particularly during a pandemic – which
        didn't happen until July of 2020. They have shown a lot of faith in HOPR and we appreciate that :).
      </>
    ),
  },
  {
    title: 'How did HOPR come to Binance’s attention and when?',
    paragraph: (
      <>
        Sebastian and Robert first met the Binance team during the Ethereum Community Conference (EthCC) in February
        2019 in Paris. We learned about their Fellowship that supports open source projects and were glad to be chosen
        for their{' '}
        <a
          className="text-color-high underline"
          href="https://www.binance.com/en/blog/334567549874987008/Binance-Labs-Fellow-HOPR-an-Incentivized-Privacy-Messaging-Protocol"
          target="_blank"
          rel="noopener noreferrer"
        >
          Binance-X Fellowship
        </a>
        .
      </>
    ),
  },
  // {
  //   title: 'What does Binance get in return for contributing?',
  //   paragraph: `We can't share details but the early contributors of the project (including Binance Labs) were purchasing HOPR tokens.`,
  // },
  // {
  //   title: 'Why did HOPR only raise a $1M funding round? Are you still accepting funds?',
  //   paragraph: `We didn't want to take more money than was needed, so we have been very strategic in the funding amount and who we have accepted funding from. We are very excited about our current group of funders and apologize for turning down many others. It's about finding the right fit. While we are not seeking more funding right now, we can never say never.`,
  // },
  {
    title: 'How do I become part of the HOPR team? Are you hiring?',
    paragraph: (
      <>
        Thanks for your interest - can you please apply via the form at{' '}
        <a
          className="text-color-high underline"
          href="https://docs.google.com/forms/d/e/1FAIpQLSeVTRBn_FGOCH3CXJqvGen95cNSCzJNJQXXep0HrU9UWe61CQ/viewform"
          target="_blank"
          rel="noopener noreferrer"
        >
          hoprnet.org/quest
        </a>
        ? Our jobs are also listed{' '}
        <a className="text-color-high underline" href="/who-is-HOPR#jobs">
          here
        </a>
        .
      </>
    ),
  },
]

export default FAQ
