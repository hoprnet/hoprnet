// Next.js bundler runs in ESM mode whereas
// Next.js web server runs in CommonJS mode
// @TODO remove package.json once Next.js web server supports ESM mode
export default {
  env: {
    NEXT_PUBLIC_GIT_COMMIT: process.env.HOPRD_GIT_COMMIT
  }
}
