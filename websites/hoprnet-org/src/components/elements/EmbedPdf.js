import 'react-pdf/dist/Page/AnnotationLayer.css'
import React from 'react'
import { SizeMe } from 'react-sizeme'
import { Document, Page, pdfjs } from 'react-pdf'
pdfjs.GlobalWorkerOptions.workerSrc = `//cdnjs.cloudflare.com/ajax/libs/pdf.js/${pdfjs.version}/pdf.worker.js`

class EmbedPdf extends React.Component {
  constructor(props) {
    super(props)

    this.state = {
      pageNumber: 1,
      numPages: null,
    }
  }

  onDocumentLoadSuccess = ({ numPages }) => {
    this.setState({ numPages })
  }

  previousPage = () => {
    const { pageNumber } = this.state

    this.setState({
      pageNumber: pageNumber - 1,
    })
  }

  nextPage = () => {
    const { pageNumber } = this.state

    this.setState({
      pageNumber: pageNumber + 1,
    })
  }

  render() {
    const { src } = this.props
    const { pageNumber, numPages } = this.state

    return (
      <SizeMe
        monitorHeight
        refreshRate={500}
        refreshMode={'debounce'}
        render={({ size }) => (
          <div className="pdfContainer">
            <Document file={src} onLoadSuccess={this.onDocumentLoadSuccess}>
              <Page width={size.width} pageNumber={pageNumber} />
            </Document>
            <div className="mb-24 pdfPages">
              <p>
                Page {pageNumber || (numPages ? 1 : '--')} of {numPages || '--'}
              </p>
              <button type="button" disabled={pageNumber <= 1} onClick={this.previousPage}>
                Previous
              </button>
              <button type="button" disabled={pageNumber >= numPages} onClick={this.nextPage}>
                Next
              </button>
            </div>
          </div>
        )}
      />
    )
  }
}

export default EmbedPdf
