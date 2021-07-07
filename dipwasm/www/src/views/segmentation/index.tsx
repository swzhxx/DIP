import React, { MouseEventHandler, useRef, useState, MouseEvent } from 'react'
import { Button } from 'antd'
import _ from 'lodash'
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
  })

  const onLoad = () => {
    if (canvasRef.current == null) return
    let ctx: CanvasRenderingContext2D | null =
      canvasRef.current.getContext('2d')
    if (ctx == null) return
    ctx.drawImage(imgRef?.current as HTMLImageElement, 0, 0)
    // let rect: DOMRect = canvasRef?.current?.getBoundingClientRect()
  }

  const onMouseDown = (e: MouseEvent) => {
    if (state.status == NONE) return
    setState({ ...state, isDown: true })
  }
  const onMouseMove = (e: MouseEvent) => {
    if (state.status == NONE || state.isDown == false) return
    console.log(e)
    const nativeEvent = e.nativeEvent
    const { offsetX, offsetY } = nativeEvent
    if (state.status === BACKGROUND) {
      state.backgrounds.set(`${offsetX},${offsetY}`, 1)
    }
    if (state.status === FRONT) {
      state.front.set(`${offsetX},${offsetY}`, 1)
    }
  }
  const onMouseUp = (e: MouseEvent) => {
    setState({ ...state, isDown: false })
    console.log(e)
  }

  const map2Unit32Array = (hashmap: Map<string, string>) => {
    let points = Array.from(hashmap.keys()).map((key: string) => {
      const [x, y] = key.split(',')
      return [Number(y), Number(x)]
    })
    let ps: number[] = _.flatten(points)
    return Uint32Array.from(ps)
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
      <Button onClick={() => setState({ ...state, status: FRONT })}>
        前景
      </Button>
      <Button onClick={() => setState({ ...state, status: BACKGROUND })}>
        背景
      </Button>
      <Button>分割</Button>
    </div>
  )
}
