import React from 'react'
import { Document, Page, pdfjs } from 'react-pdf'
pdfjs.GlobalWorkerOptions.workerSrc = `//cdnjs.cloudflare.com/ajax/libs/pdf.js/${pdfjs.version}/pdf.worker.js`;
import 'react-pdf/dist/Page/AnnotationLayer.css';

class EmbedPdf extends React.Component {
  constructor(props) {
    super(props);
    this.state = {
      pageNumber: 1,
      numPages: null
    }
    this.onDocumentLoadSuccess = this.onDocumentLoadSuccess.bind(this);
  }

  onDocumentLoadSuccess({ numPages }) {
    this.setState({ numPages });
  }

  render() {
    const { src } = this.props;
    return (
      <Document
        file={src}
        onLoadSuccess={this.onDocumentLoadSuccess}
      >
        <Page pageNumber={this.state.pageNumber} />
      </Document>
    )
  }
}

export default EmbedPdf
