import React, { useState, useRef, useEffect } from 'react'
import House1 from '@/assets/image/1.png'
import House2 from '@/assets/image/2.png'
import { initImageElement, imageElementToImageData } from '@/utils/image'
import { Slam } from '@/slam'

import {
  ArcRotateCamera,
  Color4,
  Engine,
  PointsCloudSystem,
  Scene,
  FreeCamera,
  Vector3,
} from 'babylonjs'

type CanvasRect = {
  width: number
  height: number
}

export default (): JSX.Element => {
  const [images, setImages] = useState<Array<ImageData>>([])
  const canvas3dEl = useRef<HTMLCanvasElement>(null)

  const [canvasReact, setCanvasRect] = useState<CanvasRect>({
    width: 0,
    height: 0,
  })

  useEffect(() => {
    ;(async () => {
      let house1 = await initImageElement(House1)
      let house2 = await initImageElement(House2)
      let house1ImageData = imageElementToImageData(house1)
      let house2ImageData = imageElementToImageData(house2)
      if (house1ImageData == undefined || house2ImageData == undefined) {
        return
      }
      setImages([house1ImageData, house2ImageData])
      setCanvasRect({ width: house1.width, height: house1.height })
      recover([house1ImageData, house2ImageData])
    })()
  }, [])

  const recover = (images: Array<ImageData>) => {
    let refImage = images[0]

    let recover = new Slam.Recover3D(images)
    let points = recover.recover_3d_without_color(320)
    // let depths = recover.get_normalize_depth()
    console.log(`points`, points)
    // console.log(`depths`, depths)
    const el = canvas3dEl.current
    if (!el) {
      return
    }
    const engine = new Engine(el, true)
    const createScene = (): Scene => {
      let scene = new Scene(engine)
      // var camera = new ArcRotateCamera(
      //   'Camera',
      //   0,
      //   0,
      //   40,
      //   new Vector3(0, 0, 1),
      //   scene
      // )
      // camera.setPosition(new Vector3(0, 0, 0))
      const camera = new FreeCamera('camera1', new Vector3(0, 0, 0), scene)
      camera.inputs.addMouseWheel()
      camera.setTarget(new Vector3(0, 0, 1))
      camera.attachControl(true)

      var pcs = new PointsCloudSystem('pcs', 2, scene)

      var setPoint = function (
        particle: { position: Vector3; color: Color4 },
        i: number,
        s: any
      ) {
        let z = points[i * 5 + 2]
        let x = points[i * 5]
        let y = -points[i * 5 + 1]
        let u = parseInt(points[i * 5 + 3] + '')
        let v = parseInt(points[i * 5 + 4] + '')
        let index = v * refImage.width + u
        let r = refImage.data[index * 4] / 255
        let g = refImage.data[index * 4 + 1] / 255
        let b = refImage.data[index * 4 + 2] / 255
        // console.log(`z`)
        particle.position = new Vector3(x, y, z)
        particle.color = new Color4(
          // depths[i] / 255,
          // depths[i] / 255,
          // depths[i] / 255,
          r,
          g,
          b,
          1
        )

        // //深度图
        // particle.position = new Vector3(
        //   parseInt(i / refImage.height + ''),
        //   parseInt(i / refImage.width + ''),
        //   0
        // )
        // particle.color = new Color4(z , z , z , 255)

        // console.log(`partical color`, particle.color)
        //diff between using i and s can be seen by removing comment marker from line 14
        // particle.position = new Vector3(
        //   recoverInfo.points3d[3 * i] * 10000,
        //   recoverInfo.points3d[3 * i + 1] * 10000,
        //   recoverInfo.points3d[3 * i + 2] * 10000
        // )
        // //particle.position = new BABYLON.Vector3(particle.groupId * 0.5 + 0.25 * Math.random(), s / 5000, 0.25 * Math.random());
        // particle.color = new Color4(
        //   recoverInfo.colors[4 * i],
        //   recoverInfo.colors[4 * i + 2],
        //   recoverInfo.colors[4 * i + 3],
        //   recoverInfo.colors[4 * i + 4]
        // )
      }
      pcs.addPoints(points.length / 5, setPoint)
      pcs.buildMeshAsync()
      return scene
    }
    const scene = createScene()
    engine.runRenderLoop(function () {
      scene.render()
    })
  }
  return <canvas width={800} height={800} ref={canvas3dEl}></canvas>
}
