import React, { useState, useRef, ChangeEvent, MouseEvent } from 'react'

type Style = {
  width?: number
  height?: number
}

const colors = ['red', 'green', 'blue']

export default (): JSX.Element => {
  const canvasEl = useRef<HTMLCanvasElement>(null)
  const [style, setStyle] = useState<Style>({})
  const [parallelLines, setParallelLines] = useState<Array<number>>([])
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
    // context.
  }
  // const onCanvasMouseUp = (event: MouseEvent) => {}
  return (
    <div>
      <input type='file' accept='image/*' onChange={handleDisplayImage} />
      <button onClick={() => drawParallelLines(0)}>第一组平行线</button>
      <button onClick={() => drawParallelLines(1)}>第二组平行线</button>
      <button onClick={() => drawParallelLines(2)}>第三组平行线</button>
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
