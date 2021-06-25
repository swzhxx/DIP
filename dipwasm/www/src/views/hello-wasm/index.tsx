import React, { useRef, useState } from 'react'
import { Button } from 'antd'
import { wasm } from '@/wasm'

// let wasm = await import('@dipwasm')

const HelloWasm = (props: any): JSX.Element => {
  const handleClick = () => {
    return wasm.wasmalert('wasmlog banana')
  }

  const [showPapeerNoise, setShowPapperNoise] = useState<Boolean>(false)

  const handleSliceSharedRef = () => {
    setShowPapperNoise(false)
    let a = new Uint8Array(200)
    wasm.takeNumberSliceBySharedRef(a)
    wasm.wasmalert('请查看Console')
    console.log(a)
  }

  const canvas = useRef<HTMLCanvasElement>(null)
  const canvas2 = useRef<HTMLCanvasElement>(null)
  const img = useRef<HTMLImageElement>(null)

  const handleShareImageData = () => {
    setShowPapperNoise(false)
    let width = 100
    let height = 100
    let total = width * height * 4
    let imageData = new ImageData(new Uint8ClampedArray(total), width, height)
    imageData = wasm.makeImageData(imageData)
    canvas?.current?.getContext('2d')?.putImageData(imageData, 0, 0)
  }

  const makePapperNoise = () => {
    setShowPapperNoise(true)
    setTimeout(() => {
      let ctx = canvas2?.current?.getContext('2d')
      if (!ctx) {
        return
      }
      let imgElement: HTMLImageElement = img.current as HTMLImageElement
      ctx.drawImage(imgElement, 0, 0)
      let imageData = wasm.letsPapperNoise(ctx.getImageData(0, 0, 512, 512))
      ctx.putImageData(imageData, 0, 0)
    })
  }

  return (
    <div>
      {[
        <Button key='1' onClick={handleClick}>
          Hello Wasm
        </Button>,
        <Button
          key='2'
          onClick={handleSliceSharedRef}
          style={{ marginLeft: '5px' }}
        >
          Slice Shared Ref
        </Button>,
        <Button
          key='3'
          onClick={handleShareImageData}
          style={{ marginLeft: '5px' }}
        >
          makeCanvasGray
        </Button>,
        <Button style={{ marginLeft: '5px' }} onClick={makePapperNoise}>
          Let Me Try Papper Noise
        </Button>,
      ]}
      <div
        style={{ marginTop: '10px', display: 'flex', justifyContent: 'center' }}
      >
        <div style={{ display: showPapeerNoise ? 'inline-block' : 'none' }}>
          <h3>Origin:</h3>
          <img
            ref={img}
            src={`${process.env.PUBLIC_URL}/images/lena_512.jpg`}
            style={{ width: '512px', height: '512px' }}
          ></img>
        </div>
        {!showPapeerNoise ? (
          <canvas
            key='2'
            ref={canvas}
            width='800px'
            height='800px'
            style={{
              border: '1px solid gray',
            }}
          ></canvas>
        ) : (
          <>
            <div style={{ display: 'inline-block' }}>
              <h3>Papper Noise:</h3>
              <canvas ref={canvas2} width='512px' height='512px'></canvas>
            </div>
          </>
        )}
      </div>
    </div>
  )
}

export default HelloWasm
