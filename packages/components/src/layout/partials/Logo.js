import React from 'react'
import classNames from 'classnames'
import { Link } from 'react-router-dom'
import Image from '../../elements/Image'

const isCompany = process.env.REACT_APP_IS_COMPANY === 'TRUE'

const Logo = ({ className, ...props }) => {
  const classes = classNames('brand', className)

  return (
    <div {...props} className={classes}>
      <Link to="/">
        <div style={{ display: 'flex' }}>
          <div style={{ width: '100%' }}>
            <Image
              src={require('../../assets/images/logo_gradient.png')}
              alt="HOPR Logo"
              height="auto"
              width="100px"
              className="mr-12"
            />
          </div>
          <span className="h4 p-0 m-0">{isCompany ? 'HOPR Services' : undefined}</span>
        </div>
      </Link>
    </div>
  )
}

export default Logo
