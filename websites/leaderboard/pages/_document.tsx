import NextDocument, { Html, Head, Main } from 'next/document'

class Document extends NextDocument {
  render() {
    return (
      <Html>
        <Head>
          <script defer src="https://panther.hoprnet.org/script.js" data-site="MPSLSLOQ" />
        </Head>
        <body>
          <Main />
        </body>
      </Html>
    )
  }
}

export default Document
