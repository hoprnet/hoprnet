import React from 'react'
import { Document, Page, pdfjs } from 'react-pdf'
pdfjs.GlobalWorkerOptions.workerSrc = `//cdnjs.cloudflare.com/ajax/libs/pdf.js/${pdfjs.version}/pdf.worker.js`;
import 'react-pdf/dist/Page/AnnotationLayer.css';

const EmbedPdf = ({ src, ...props }) => {
  return (
    <Document file={src}>
      <Page />
    </Document>
  )
}

export default EmbedPdf
