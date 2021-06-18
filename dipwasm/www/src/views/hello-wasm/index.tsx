import React from 'react'
import { Button } from 'antd'
// import { wasmlog } from 'dipwasm'
const HelloWasm: React.FC<Object> = (props) => {
  const handleClick = () => {
    return import('@/../pkg/index.js').then((module) => {
      module.wasmalert('wasmlog banana')
    })
  }
  return <Button onClick={handleClick}>Hello Wasm</Button>
}

export default HelloWasm
