/** @type {import('next').NextConfig} */
module.exports = {
  reactStrictMode: true,
  env: {
    NEXT_PUBLIC_GIT_COMMIT: process.env.HOPRD_GIT_COMMIT
  }
}
