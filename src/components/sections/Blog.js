import React, { useEffect } from 'react'
import classNames from 'classnames'
import { SectionProps } from '../../utils/SectionProps'
import SectionHeader from './partials/SectionHeader'

const propTypes = {
  ...SectionProps.types,
}

const defaultProps = {
  ...SectionProps.defaults,
}

const fetchFeed = async () => {
  return fetch('https://medium.com/feed/@SCBuergel', {
    headers: {
      'Access-Control-Allow-Origin': '*',
    },
  }).then(res => res.json())
}

const Blog = props => {
  const {
    id,
    className,
    topOuterDivider,
    bottomOuterDivider,
    topDivider,
    bottomDivider,
    hasBgColor,
    invertColor,
  } = props

  const outerClasses = classNames(
    'blog section center-content',
    topOuterDivider && 'has-top-divider',
    bottomOuterDivider && 'has-bottom-divider',
    hasBgColor && 'has-bg-color',
    invertColor && 'invert-color',
    className
  )

  const innerClasses = classNames(
    'blog-inner section-inner',
    topDivider && 'has-top-divider',
    bottomDivider && 'has-bottom-divider'
  )

  const sectionHeader = {
    title: 'Blog',
    paragraph: undefined,
  }

  useEffect(() => {
    fetchFeed().then(console.log)
  }, [])

  return (
    <section id={id} className={outerClasses}>
      <div className="container">
        <div className={innerClasses}>
          <SectionHeader data={sectionHeader} className="center-content" />
          {/* <div
            id="retainable-rss-embed"
            data-rss="https://medium.com/feed/@SCBuergel"
            data-maxcols="3"
            data-layout="grid"
            data-poststyle="inline"
            data-readmore="Read the rest"
            data-buttonclass="btn btn-primary"
            data-offset="-100"
          ></div> */}
        </div>
      </div>
    </section>
  )
}

Blog.propTypes = propTypes
Blog.defaultProps = defaultProps

export default Blog
