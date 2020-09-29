import { ChakraProvider, extendTheme } from '@chakra-ui/core'
import { AppProps } from 'next/app'
import '../styles/globals.css'

const overrides = extendTheme({
  styles: {
    global: () => ({
      body: {
        fontFamily: 'monospace',
        padding: 0,
        margin: 0,
      },
      a: {
        color: '#0000B4',
        fontWeight: 'bold',
        textDecoration: 'none',
      },
    }),
  },
})

function App({ Component, pageProps }: AppProps): React.ReactNode {
  return (
    <ChakraProvider resetCSS={true} theme={overrides}>
      <Component {...pageProps} />
    </ChakraProvider>
  )
}

export default App
