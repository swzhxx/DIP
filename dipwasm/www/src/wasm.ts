let Splat: typeof import('@pkg/splatoon')
import('@pkg/splatoon').then((module) => {
  Splat = module
})
export { Splat }
