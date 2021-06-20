import type DipWasm from '@pkg/index'
let wasm: DipWasm = {}

import('@pkg/index').then((module) => {
  wasm = module
})

export { wasm }
