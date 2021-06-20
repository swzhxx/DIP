let wasm: any

import('@pkg/index').then((module) => {
  wasm = module
})

export { wasm }
