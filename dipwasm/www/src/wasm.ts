let wasm: typeof import('@pkg/index')
import('@pkg/index').then((module) => {
  wasm = module
})
export { wasm }
