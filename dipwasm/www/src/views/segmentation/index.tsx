import React, { MouseEventHandler, useRef, useState, MouseEvent } from 'react'
import { Button } from 'antd'
import _ from 'lodash'
import { Splat } from '@/wasm'
const FRONT = 1
const BACKGROUND = 2
const NONE = 3

export default (): JSX.Element => {
  const imgRef = useRef<HTMLImageElement>(null)
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const [state, setState] = useState<any>({
    status: 3,
    isDown: false,
    fronts: new Map(),
    backgrounds: new Map(),
    imageData: undefined,
  })

  const onLoad = () => {
    if (canvasRef.current == null) return
    let ctx: CanvasRenderingContext2D | null =
      canvasRef.current.getContext('2d')
    if (ctx == null) return
    ctx.drawImage(imgRef?.current as HTMLImageElement, 0, 0)
    setState({ ...state, imageData: ctx.getImageData(0, 0, 512, 512).data })
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
    console.log(e)
    const nativeEvent = e.nativeEvent
    const { offsetX, offsetY } = nativeEvent
    let _ctx = canvasRef?.current?.getContext('2d')
    if (!(_ctx instanceof CanvasRenderingContext2D)) {
      throw new Error('Failed to get 2D context')
    }
    const ctx: CanvasRenderingContext2D = _ctx
    if (ctx == null) return
    if (state.status === BACKGROUND) {
      state.backgrounds.set(`${offsetX},${offsetY}`, 1)
      ctx.strokeStyle = `rgb(255,0,0)`
    }
    if (state.status === FRONT) {
      state.fronts.set(`${offsetX},${offsetY}`, 1)
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
    const imageData = state.imageData
    const frontPixels = map2Unit32Array(state.fronts)
    const backgroundPixels = map2Unit32Array(state.backgrounds)
    Splat.graphCuts(imageData, 512, 512, frontPixels, backgroundPixels)
  }
  return (
    <div>
      <img
        ref={imgRef}
        onLoad={onLoad}
        src={`${process.env.PUBLIC_URL}/images/lena_512.jpg`}
        style={{ width: '512px', height: '512px', display: 'none' }}
      ></img>
      <canvas
        width='512'
        height='512'
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
      <div>
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
