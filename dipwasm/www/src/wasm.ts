let wasm: typeof import('@pkg/splatoon')
import('@pkg/splatoon').then((module) => {
  wasm = module
})
export { wasm }
