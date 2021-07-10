import React, { MouseEventHandler, useRef, useState, MouseEvent } from 'react'
import { Button } from 'antd'
import _ from 'lodash'
import { Splat } from '@/wasm'
const FRONT = 1
const BACKGROUND = 2
const NONE = 3
const WIDTH = 512
const HEIGHT = 512
export default (): JSX.Element => {
  const imgRef = useRef<HTMLImageElement>(null)
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const [state, setState] = useState<any>({
    status: 3,
    isDown: false,
    // fronts: new Map(),
    // backgrounds: new Map(),
    imageData: undefined,
  })

  const onLoad = () => {
    if (canvasRef.current == null) return
    let ctx: CanvasRenderingContext2D | null =
      canvasRef.current.getContext('2d')
    if (ctx == null) return
    // ctx.drawImage(imgRef?.current as HTMLImageElement, 0, 0)
    setState({
      ...state,
      imageData: ctx.getImageData(0, 0, WIDTH, HEIGHT).data,
    })
    // let rect: DOMRect = canvasRef?.current?.getBoundingClientRect()
  }

  const onMouseDown = (e: MouseEvent) => {
    if (state.status == NONE) return
    setState({ ...state, isDown: true })
    const ctx = canvasRef?.current?.getContext('2d')
    ctx?.beginPath()
    ctx?.moveTo(e.nativeEvent.offsetX, e.nativeEvent.offsetY)
  }
  const onMouseMove = (e: MouseEvent) => {
    if (state.status == NONE || state.isDown == false) return
    // console.log(e)
    const nativeEvent = e.nativeEvent
    const { offsetX, offsetY } = nativeEvent
    let _ctx = canvasRef?.current?.getContext('2d')
    if (!(_ctx instanceof CanvasRenderingContext2D)) {
      throw new Error('Failed to get 2D context')
    }
    const ctx: CanvasRenderingContext2D = _ctx
    ctx.lineWidth = 5
    if (ctx == null) return
    if (state.status === BACKGROUND) {
      // state.backgrounds.set(`${offsetX},${offsetY}`, 1)
      ctx.strokeStyle = `rgb(255,0,0)`
    }
    if (state.status === FRONT) {
      // state.fronts.set(`${offsetX},${offsetY}`, 1)
      ctx.strokeStyle = `rgb(0,255,0)`
    }

    ctx?.lineTo(offsetX, offsetY)
    ctx?.stroke()
    // ctx?.closePath()
    // ctx?.beginPath()
    // ctx?.lineTo(offsetX, offsetY)
    // ctx?.moveTo(offsetX, offsetY)
  }
  const onMouseUp = (e: MouseEvent) => {
    setState({ ...state, isDown: false })
    console.log(e)
    let ctx = canvasRef?.current?.getContext('2d')
    ctx?.closePath()
  }

  const map2Unit32Array = (hashmap: Map<string, string>) => {
    let points = Array.from(hashmap.keys()).map((key: string) => {
      const [x, y] = key.split(',')
      return [Number(y), Number(x)]
    })
    let ps: number[] = _.flatten(points)
    return Uint32Array.from(ps)
  }

  const handleGraphCuts = () => {
    let width = WIDTH
    let height = HEIGHT
    let fronts = new Map()
    let backgrounds = new Map()
    let ctx = canvasRef?.current?.getContext('2d')
    let canvasData = ctx?.getImageData(0, 0, width, height)
    canvasData?.data.forEach((val, index) => {
      if (val !== 255) {
        return
      } else {
        let offset = index % 4
        let y: number = Math.floor(index / 4 / width)
        let x = Math.floor((index / 4) % width)
        if (offset == 0) {
          backgrounds.set(`${x},${y}`, 1)
        }
        if (offset == 1) {
          fronts.set(`${x},${y}`, 1)
        }
      }
    })

    const imageData = state.imageData
    const frontPixels = map2Unit32Array(fronts)
    const backgroundPixels = map2Unit32Array(backgrounds)
    let gaussianImageData = Splat.splatGaussianFilter(
      imageData,
      width,
      height,
      11
    )
    let mincuts = Splat.graphCuts(
      gaussianImageData.data,
      height,
      width,
      frontPixels,
      backgroundPixels
    )

    let _imageData = new Array(height * width * 4).fill(255)

    // ctx?.putImageData(_imageData, 0, 0)
    let getIndex = (y: number, x: number): number => {
      return (y * width + x) * 4
    }
    mincuts.reduce((prev, val, index) => {
      if (index % 2 == 1) {
        let y = prev
        let x = val
        //TODO
        let index = getIndex(y, x)
        let pixel = imageData.slice(index, index + 4)
        _imageData[index] = pixel[0]
        _imageData[index + 1] = pixel[1]
        _imageData[index + 2] = pixel[2]
        _imageData[index + 3] = pixel[3]

        return -1
      } else {
        return val
      }
    }, -1)

    let cutImageData = new ImageData(
      new Uint8ClampedArray(_imageData),
      width,
      height
    )
    ctx?.putImageData(cutImageData, 0, 0)
  }
  return (
    <div>
      <div style={{ position: 'relative', display: 'inline-block' }}>
        <img
          ref={imgRef}
          onLoad={onLoad}
          src={`${process.env.PUBLIC_URL}/images/lena_512.jpg`}
          style={{
            width: `${WIDTH}px`,
            height: `${HEIGHT}px`,
          }}
        ></img>
        <canvas
          style={{ position: 'absolute', top: '0', left: '0' }}
          width={WIDTH}
          height={HEIGHT}
          ref={canvasRef}
          onMouseDown={(e: MouseEvent) => {
            onMouseDown(e)
          }}
          onMouseMove={(e: MouseEvent) => {
            onMouseMove(e)
          }}
          onMouseUp={(e: MouseEvent) => {
            onMouseUp(e)
          }}
        ></canvas>
      </div>
      <div style={{ marginTop: '10px' }}>
        <Button onClick={() => setState({ ...state, status: FRONT })}>
          前景
        </Button>
        <Button onClick={() => setState({ ...state, status: BACKGROUND })}>
          背景
        </Button>
        <Button onClick={handleGraphCuts}>分割</Button>
      </div>
    </div>
  )
}
