const path = require('path')
const { override, addWebpackAlias, addWebpackPlugin } = require('customize-cra')
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin')
module.exports = override(
  addWebpackPlugin(
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, './../'),
      outDir: path.resolve(__dirname, './pkg'),
    })
  ),
  addWebpackAlias({
    '@': path.resolve(__dirname, './src'),
    dipwasm: path.resolve(__dirname, './../pkg'),
  }),
  (config, env) => {
    const wasmExtensionRegExp = /\.wasm$/

    config.resolve.extensions.push('.wasm')

    config.module.rules.forEach((rule) => {
      ;(rule.oneOf || []).forEach((oneOf) => {
        if (oneOf.loader && oneOf.loader.indexOf('file-loader') >= 0) {
          // make file-loader ignore WASM files
          oneOf.exclude.push(wasmExtensionRegExp)
        }
      })
    })
    config.resolve.plugins = config.resolve.plugins.filter(
      (plugin) => plugin.__proto__.constructor.name !== 'ModuleScopePlugin'
    )

    // add a dedicated loader for WASM
    config.module.rules.push({
      test: wasmExtensionRegExp,
      include: path.resolve(__dirname, 'src'),
      use: [{ loader: require.resolve('wasm-loader'), options: {} }],
    })

    return config
  }
)
