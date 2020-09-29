import "../styles/globals.css";
import { AppProps } from "next/app";
import { ThemeProvider } from "@chakra-ui/core";

const App = ({ Component, pageProps }) => {
  return (
    <ThemeProvider>
      <Component {...pageProps} />
    </ThemeProvider>
  );
};

export default App;
