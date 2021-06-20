import React from 'react'
import { Button } from 'antd'
import { wasm } from '@/wasm'

// let wasm = await import('@dipwasm')

const HelloWasm = (props: any): JSX.Element => {
  const handleClick = () => {
    return wasm.wasmalert('wasmlog banana')
  }
  const handleSliceSharedRef = () => {
    let a = new Uint8Array(200)
    wasm.take_number_slice_by_shared_ref(a)
    console.log(a)
  }
  return (
    <>
      {[
        <Button key='1' onClick={handleClick}>
          Hello Wasm
        </Button>,
        <Button key='2' onClick={handleSliceSharedRef}>
          Slice Shared Ref
        </Button>,
      ]}
    </>
  )
}

export default HelloWasm
