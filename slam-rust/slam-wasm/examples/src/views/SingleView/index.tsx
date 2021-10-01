import React, {
  useState,
  useRef,
  ChangeEvent,
  MouseEvent,
  useEffect,
} from 'react'
import { Slam } from '@/slam'
// import { group } from 'console'
import stf from '@/assets/image/1.jpg'
import {
  Scene,
  Engine,
  PointsCloudSystem,
  PointLight,
  ArcRotateCamera,
  Vector3,
  Color4,
} from 'babylonjs'
console.log(`stf`, stf)
type Style = {
  width?: number
  height?: number
}

type Recover = {
  points3d: Float64Array
  colors: Uint8Array
}

const colors = ['red', 'green', 'blue']

export default (): JSX.Element => {
  const canvasEl = useRef<HTMLCanvasElement>(null)
  const canvas3dEl = useRef<HTMLCanvasElement>(null)
  // 结构恢复的场景信息
  const [recoverInfo, setRecoverInfo] = useState<Recover>({
    points3d: new Float64Array(),
    colors: new Uint8Array(),
  })
  // 图像信息
  const [imageData, setImageData] = useState<HTMLImageElement | null>(null)
  // 样式信息
  const [style, setStyle] = useState<Style>({
    width: imageData?.width,
    height: imageData?.height,
  })
  const [isRecovered, setIsRecovered] = useState<boolean>(false)
  const [parallelLines, setParallelLines] = useState<Array<number>>([
    674, 1826, 2456, 1060, 1094, 1340, 1774, 1086, 674, 1826, 126, 1056, 2456,
    1060, 1940, 866, 1094, 1340, 1080, 598, 1774, 1086, 1840, 478,
  ])

  useEffect(() => {
    let stfImageData = new Image()
    stfImageData.onload = () => {
      setImageData(stfImageData)
      setStyle({ width: stfImageData.width, height: stfImageData.height })
    }
    stfImageData.src = stf
    // document.body.append(stfImageData)
    // refreshCanvas()
  }, [])
  useEffect(() => {
    setTimeout(refreshCanvas)
  }, [imageData, canvasEl, parallelLines])

  useEffect(() => {
    if (!isRecovered) {
      return
    }
    renderPCS()
  }, [isRecovered])

  const refreshCanvas = () => {
    const context = getContext()
    if (!context || !imageData) return
    context.clearRect(0, 0, style.width || 0, style.height || 0)
    context.drawImage(
      imageData,
      0,
      0,
      imageData.width || 0,
      imageData.height || 0
    )
    drawParallLine()
  }

  const handleUploadImage = (event: ChangeEvent) => {
    let files = (event.target as HTMLInputElement).files
    if (files == null) {
      return
    }
    let file: File = files[0]
    let reader = new FileReader()
    reader.readAsDataURL(file)
    reader.onload = (res) => {
      // drawOriginImage(reader.result as string)
      let dataURL = reader.result as string
      let image = new Image()
      image.onload = () => {
        setImageData(image)
        setStyle({ width: image.width, height: image.height })
        setParallelLines([])
      }
      image.src = dataURL
    }
  }

  const drawParallLine = () => {
    const context = getContext()
    if (context == null) {
      return
    }
    for (let i = 1; i < parallelLines.length; i++) {
      let j = i + 1

      if (j % 2 == 1) {
        continue
      }
      if (j % 2 == 0) {
        const offsetX = parallelLines[i - 1]
        const offsetY = parallelLines[i]

        let groupParallLineIndex = Math.floor(j / 8)
        if (j % 8 == 0) {
          groupParallLineIndex--
        }
        context.beginPath()
        context.arc(offsetX, offsetY, 5, 0, Math.PI * 2, true)
        context.fillStyle = colors[groupParallLineIndex]
        context.fill()
        if (j % 4 == 0) {
          const prevX = parallelLines[i - 3]
          const prevY = parallelLines[i - 2]
          context.beginPath()
          context.moveTo(prevX, prevY)
          context.lineTo(offsetX, offsetY)
          context.lineWidth = 5
          context.strokeStyle = colors[groupParallLineIndex]
          context.stroke()
        }
      }
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

  const onCanavasMouseDown = (event: MouseEvent) => {
    console.log(event)
    const context = getContext()
    if (!context) {
      return
    }
    if (parallelLines.length >= 24) {
      return
    }
    const nativeEevent = event.nativeEvent
    const { offsetX, offsetY } = nativeEevent
    let _parallelLines = parallelLines.concat(offsetX, offsetY)
    setParallelLines(_parallelLines)
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
    let points3d = singleViewRecover.get_own_points3d()
    console.log(`points3d`, points3d)
    let colors = singleViewRecover.get_own_colors()
    if (points3d.length) {
      setRecoverInfo({
        points3d,
        colors,
      })
      setIsRecovered(true)
    }
  }

  const renderPCS = () => {
    const el = canvas3dEl.current
    if (!el) {
      return
    }
    const engine = new Engine(el, true)
    const createScene = (): Scene => {
      let scene = new Scene(engine)
      var camera = new ArcRotateCamera(
        'Camera',
        -Math.PI / 2,
        Math.PI / 3,
        8,
        new Vector3(0, 0, 0),
        scene
      )
      camera.attachControl(el, true)

      var pcs = new PointsCloudSystem('pcs', 2, scene)

      var setPoint = function (
        particle: { position: Vector3; color: Color4 },
        i: number,
        s: any
      ) {
        //diff between using i and s can be seen by removing comment marker from line 14
        particle.position = new Vector3(
          recoverInfo.points3d[3 * i] * 10000,
          recoverInfo.points3d[3 * i + 1] * 10000,
          recoverInfo.points3d[3 * i + 2] * 10000
        )
        //particle.position = new BABYLON.Vector3(particle.groupId * 0.5 + 0.25 * Math.random(), s / 5000, 0.25 * Math.random());
        particle.color = new Color4(
          recoverInfo.colors[4 * i],
          recoverInfo.colors[4 * i + 2],
          recoverInfo.colors[4 * i + 3],
          recoverInfo.colors[4 * i + 4]
        )
      }
      pcs.addPoints(10000, setPoint)
      pcs.buildMeshAsync()
      return scene
    }
    const scene = createScene()
    engine.runRenderLoop(function () {
      scene.render()
    })
  }

  return (
    <div>
      <input type='file' accept='image/*' onChange={handleUploadImage} />
      <button onClick={() => recover()}>恢复</button>
      <canvas
        onMouseDown={onCanavasMouseDown}
        // onMouseUp={onCanvasMouseUp}
        width={style.width}
        height={style.height}
        ref={canvasEl}
      ></canvas>

      {isRecovered && (
        <canvas
          width={style.width}
          height={style.height}
          ref={canvas3dEl}
        ></canvas>
      )}
    </div>
  )
}
