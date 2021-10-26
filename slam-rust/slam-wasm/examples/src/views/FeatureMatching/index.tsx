import React, { ChangeEvent, useEffect, useState, useRef } from 'react'
import { Slam } from '@/slam'
import {
  imageElementToImageData,
  fileToImageData,
  initImageElement,
} from '@/utils/image'
import House1 from '@/assets/image/1.png'
import House2 from '@/assets/image/2.png'

type CanvasRect = {
  width: number
  height: number
}

type Point = {
  x: number
  y: number
}

export default (): JSX.Element => {
  const [images, setImages] = useState<Array<ImageData>>([])
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const [isMatched, setIsMatched] = useState<boolean>(false)
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
    })()
  }, [])

  useEffect(() => {
    if (images.length != 2) {
      return
    }
    let orb = new Slam.OrbFeatureMatcher(images[0], images[1])
    orb.feature_point_matching(40, 40)
    // let matching = Slam.feature_point_matching(images[0], images[1], 80)

    drawMathingResult(
      orb.get_feature_points_1(),
      orb.get_feature_points_2(),
      orb.get_matched()
    )
  }, [images])

  const handleUploadImage = async (event: ChangeEvent) => {
    let files = (event.target as HTMLInputElement).files
    if (files == null) {
      return
    }
    if (files.length != 2) {
      alert('仅支持提交2个图片')
      return
    }
    let images = []
    for (let i = 0; i < files.length; i++) {
      let imageData = await fileToImageData(files[i])
      images.push(imageData)
    }

    setImages(images)
  }

  const drawMathingResult = (
    features1: Uint32Array,
    features2: Uint32Array,
    matched: Uint32Array
  ) => {
    console.log(`features1`, features1)
    console.log(`features2`, features2)
    console.log(`matched`, matched)
    let image1Width = images[0].width
    let image1Height = images[0].height
    let image2Width = images[0].width
    let image2Height = images[0].height
    let padding1X = 0
    let padding1Y = 0
    let padding2X = 0
    let padding2Y = 0
    if (image1Width != image2Width || image1Height != image2Height) {
      //padding
      if (image1Width - image2Width > 0) {
        padding2X = Math.floor((image1Width - image2Width) / 2)
      } else {
        padding1X = Math.floor((image2Width - image1Width) / 2)
      }

      if (image1Height - image2Height > 0) {
        padding2Y = Math.floor((image1Height - image2Height) / 2)
      } else {
        padding1Y = Math.floor((image1Height - image2Height) / 2)
      }
    }

    if (images.length != 2) {
      return
    }
    setCanvasRect({
      width: padding1X * 2 + images[0].width + padding2X * 2 + images[1].width,
      height:
        padding1Y * 2 + images[0].height + padding2Y * 2 + images[1].height,
    })
    // console.log(`images`, images[0], images[1])
    setTimeout(() => {
      let context = canvasRef.current?.getContext('2d')
      if (context == null) {
        return
      }
      context.putImageData(
        images[0],
        padding1X,
        padding1Y,
        0,
        0,
        images[0].width,
        images[0].height
      )
      context.putImageData(
        images[1],
        padding1X * 2 + images[0].width,
        padding2Y,
        0,
        0,
        images[1].width,
        images[1].height
      )

      let points: Array<Point> = []

      for (let i = 0; i < matched.length; i = i + 3) {
        let index1 = matched[i]
        let index2 = matched[i + 1]
        let point1 = {
          x: features1[index1 * 2] + padding1X,
          y: features1[index1 * 2 + 1] + padding1Y,
        }
        let point2 = {
          x: features2[index2 * 2] + images[0].width + padding1X * 2,
          y: features2[index2 * 2 + 1] + padding2Y,
        }
        points.push(point1)
        points.push(point2)
      }
      // for (let i = 0; i < features1.length; i = i + 2) {
      //   let point1 = {
      //     x: features1[i * 2] + padding1X,
      //     y: features1[i * 2 + 1] + padding1Y,
      //   }
      //   // let point2 = {
      //   //   x: features2[i * 2] + padding2X + images[0].width,
      //   //   y: features2[i * 2 + 1] + padding2Y,
      //   // }
      //   points.push(point1)
      //   // points.push(point2)
      // }

      // for (let i = 0; i < features2.length; i = i + 2) {
      //   let point2 = {
      //     x: features2[i * 2] + padding2X + images[0].width + padding1X * 2,
      //     y: features2[i * 2 + 1] + padding2Y,
      //   }
      //   points.push(point2)
      //   // points.push(point2)
      // }
      const drawFeatures = (features: Uint32Array, offset = { x: 0, y: 0 }) => {
        for (let i = 1; i < features.length; i = i + 2) {
          let x = features[i - 1]
          let y = features[i]
          context!.beginPath()
          context!.arc(x + offset.x, y + offset.y, 3, 0, Math.PI * 2)
          context!.strokeStyle = 'purple'
          context!.stroke()
        }
      }
      drawFeatures(features1)
      drawFeatures(features2, { x: images[0].width, y: 0 })

      console.log(`points`, points)
      let colors = ['red', 'blue', 'yellow', 'white', 'pink', 'aqua']
      points.forEach((p, index) => {
        // if (index >= 10) return
        if (index > 0 && index % 2 == 1) {
          context!.beginPath()
          context!.arc(
            points[index - 1].x,
            points[index - 1].y,
            5,
            0,
            Math.PI * 2
          )
          let color = colors[Math.ceil(Math.random() * 10) % 6]
          context!.fillStyle = color
          context!.fill()
          context!.beginPath()
          context!.arc(p.x, p.y, 5, 0, Math.PI * 2)
          context!.fillStyle = color
          context!.fill()
          context!.beginPath()
          context!.moveTo(points[index - 1].x, points[index - 1].y)
          context!.lineTo(p.x, p.y)
          context!.strokeStyle = color
          context!.stroke()
        }
      })
    })
  }
  return (
    <div>
      <div>
        <input
          type='file'
          accept='image/*'
          multiple
          onChange={handleUploadImage}
        />
        <button>特征点匹配</button>
      </div>
      {
        <canvas
          width={canvasReact.width}
          height={canvasReact.height}
          ref={canvasRef}
        ></canvas>
      }
    </div>
  )
}
