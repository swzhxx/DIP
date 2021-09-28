let Slam: typeof import('@pkg/slam')

import('@pkg/slam').then((module) => {
  Slam = module
})

export { Slam }
