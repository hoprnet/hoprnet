import React from 'react'
import { EmbedPdf } from '../elements'

const Pdf = document => {
  return () => <EmbedPdf src={require(`../assets/documents/${document}`)} />
}

export default Pdf
