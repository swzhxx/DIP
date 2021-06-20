import React, { useRef } from 'react'
import { Button } from 'antd'
import { wasm } from '@/wasm'

// let wasm = await import('@dipwasm')

const HelloWasm = (props: any): JSX.Element => {
  const handleClick = () => {
    return wasm.wasmalert('wasmlog banana')
  }
  const handleSliceSharedRef = () => {
    let a = new Uint8Array(200)
    wasm.takeNumberSliceBySharedRef(a)
    console.log(a)
  }

  const canvas = useRef<HTMLCanvasElement>(null)

  const handleShareImageData = () => {
    let width = 100
    let height = 100
    let total = width * height * 4
    let imageData = new ImageData(new Uint8ClampedArray(total), width, height)
    imageData = wasm.makeImageData(imageData)
    canvas?.current?.getContext('2d')?.putImageData(imageData, 0, 0)
  }

  return (
    <div>
      {[
        <Button key='1' onClick={handleClick}>
          Hello Wasm
        </Button>,
        <Button key='2' onClick={handleSliceSharedRef}>
          Slice Shared Ref
        </Button>,
        <Button key='3' onClick={handleShareImageData}>
          makeCanvasGray
        </Button>,
      ]}
      <div>
        <canvas ref={canvas} width='400' height='400'></canvas>
      </div>
    </div>
  )
}

export default HelloWasm
