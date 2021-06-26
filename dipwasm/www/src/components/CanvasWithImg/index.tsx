import React, {
  useRef,
  useImperativeHandle,
  useState,
  useCallback,
} from 'react'
export default React.forwardRef(
  (props: any, ref: React.Ref<Object>): JSX.Element => {
    const { src, canvas, ...other } = props || {}
    const canvasRef = useRef<HTMLCanvasElement>(null)
    const imgRef = useRef<HTMLImageElement>(null)
    const [originData, setOriginData] = useState<ImageData | null>(null)
    const onLoad = () => {
      if (canvasRef.current == null) return
      let ctx: CanvasRenderingContext2D | null =
        canvasRef.current.getContext('2d')
      if (ctx == null) return
      ctx.drawImage(imgRef?.current as HTMLImageElement, 0, 0)
      let rect: DOMRect = canvasRef?.current?.getBoundingClientRect()
      setOriginData(ctx.getImageData(0, 0, rect.width, rect.height))
    }

    const putImageData = (imageData: ImageData) => {
      canvasRef?.current?.getContext('2d')?.putImageData(imageData, 0, 0)
    }

    const getOriginData = useCallback(() => {
      return originData
    }, [originData])

    useImperativeHandle(ref, () => ({
      putImageData,
      getOriginData,
    }))

    return (
      <div {...other}>
        <img
          ref={imgRef}
          onLoad={onLoad}
          style={{ display: 'none' }}
          src={src}
          alt=''
        />
        <canvas
          ref={canvasRef}
          width={canvas.width}
          height={canvas.height}
        ></canvas>
      </div>
    )
  }
)
