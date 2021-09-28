import React, { useState, useRef, ChangeEvent, MouseEvent } from 'react'
import { Slam } from '@/slam'
type Style = {
  width?: number
  height?: number
}

const colors = ['red', 'green', 'blue']

export default (): JSX.Element => {
  const canvasEl = useRef<HTMLCanvasElement>(null)
  const [style, setStyle] = useState<Style>({})
  const [parallelLines, setParallelLines] = useState<Array<number>>([
    727, 332, 1012, 419, 1005, 490, 717, 394, 1079, 599, 1831, 473, 1094, 1329,
    1781, 1080, 2090, 720, 2108, 588, 2156, 746, 2174, 600,
  ])
  const [drawParallLineIndex, setDrawParallLineIndex] = useState<number | null>(
    null
  )

  const handleDisplayImage = (event: ChangeEvent) => {
    let files = (event.target as HTMLInputElement).files
    if (files == null) {
      return
    }
    let file: File = files[0]
    let reader = new FileReader()
    reader.readAsDataURL(file)
    reader.onload = (res) => {
      drawOriginImage(reader.result as string)
    }
  }

  const getContext = () => {
    let canvasElement = canvasEl.current
    if (canvasElement == null) {
      throw new Error('canvasElement is null')
    }
    let context = canvasElement.getContext('2d')
    return context
  }

  const drawOriginImage = (dataURL: string) => {
    let context = getContext()
    const img = new Image()
    img.onload = () => {
      setStyle({ width: img.width, height: img.height })
      //const ratio = img.width / img.height
      // let maxCanvasPixel = 0
      // if (ratio > 1) {
      //   maxCanvasPixel = Math.min(img.width, 1920)
      // } else {
      //   maxCanvasPixel = Math.min(img.height, 1080)
      // }
      // let canvasWidth = ratio > 1 ? maxCanvasPixel * ratio :
      context?.drawImage(img, 0, 0, img.width, img.height)
    }
    img.src = dataURL
  }

  const drawParallelLines = (index: number) => {
    setDrawParallLineIndex(index)
  }

  const onCanavasMouseDown = (event: MouseEvent) => {
    if (drawParallLineIndex == null) {
      return
    }
    console.log(event)
    const context = getContext()
    if (!context) {
      return
    }
    const nativeEevent = event.nativeEvent
    const { offsetX, offsetY } = nativeEevent
    context.beginPath()
    context.arc(offsetX, offsetY, 5, 0, Math.PI * 2, true)
    context.fillStyle = colors[drawParallLineIndex]
    context.fill()
    let _parallelLines = parallelLines.concat(offsetX, offsetY)

    if ((parallelLines.length / 2) % 2 == 1 && parallelLines.length > 1) {
      const prevX = parallelLines[parallelLines.length - 2]
      const prevY = parallelLines[parallelLines.length - 1]

      context.beginPath()
      context.moveTo(prevX, prevY)
      context.lineTo(offsetX, offsetY)
      context.lineWidth = 5
      context.strokeStyle = colors[drawParallLineIndex]
      context.stroke()
    }

    setParallelLines(_parallelLines)
    if ((_parallelLines.length / 2 / 2) % 2 == 0) {
      let next = _parallelLines.length / 2 / 2 / 2
      setDrawParallLineIndex(next)
    }

    // context.
  }

  const recover = () => {
    let ctx = getContext()
    if (!ctx) {
      return
    }
    let image = ctx.getImageData(0, 0, style.width || 0, style.height || 0)
    let singleViewRecover = new Slam.WrapperSingleViewRecover()
    singleViewRecover.single_view_recover(
      image,
      new Float64Array(parallelLines)
    )

    let point3ds = singleViewRecover.get_own_points3d()
    let colors = singleViewRecover.get_own_colors()

    console.log(point3ds, colors)
  }
  // const onCanvasMouseUp = (event: MouseEvent) => {}
  return (
    <div>
      <input type='file' accept='image/*' onChange={handleDisplayImage} />
      <button onClick={() => drawParallelLines(0)}>选择平行线</button>
      <button onClick={() => recover()}>恢复</button>
      <canvas
        onMouseDown={onCanavasMouseDown}
        // onMouseUp={onCanvasMouseUp}
        width={style.width}
        height={style.height}
        ref={canvasEl}
      ></canvas>
    </div>
  )
}
