import React from 'react'
import SectionHeader from '../sections/partials/SectionHeader'
import GenericSection from '../sections/GenericSection'
import Substack from '../elements/Substack'

const Contact = props => {
  return (
    <GenericSection {...props}>
      <div className="container-sm">
        <SectionHeader
          data={{
            title: 'Get All Our Latest Updates!',
            paragraph: undefined,
          }}
          className="center-content"
        />
        <Substack />
      </div>
    </GenericSection>
  )
}

export default Contact
