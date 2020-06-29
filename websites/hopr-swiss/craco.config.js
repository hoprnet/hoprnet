const path = require('path')
const { getLoader, loaderByName } = require('@craco/craco')
const absolutePath = path.join(__dirname, '../../packages/components')
module.exports = {
  webpack: {
    alias: {},
    plugins: [],
    configure: (webpackConfig, { env, paths }) => {
      const { isFound, match } = getLoader(webpackConfig, loaderByName('babel-loader'))
      if (isFound) {
        const include = Array.isArray(match.loader.include) ? match.loader.include : [match.loader.include]
        match.loader.include = include.concat[absolutePath]
      }
      return webpackConfig
    },
  },
}
