import React, { ChangeEvent, useEffect, useState } from 'react'
import { Slam } from '@/slam'
import House1 from '@/assets/image/image1.jpg'
import House2 from '@/assets/image/image2.jpg'

const fileToImageELement = (file: File): Promise<HTMLImageElement> => {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => {
      const image = new Image()
      image.onload = () => {
        resolve(image)
      }
      image.src = reader.result as string
    }
  })
}

const imageElementToImageData = (imageElement: HTMLImageElement) => {
  const canvas = document.createElement('canvas')
  const context = canvas.getContext('2d')
  if (!context) return
  context.drawImage(imageElement, 0, 0, imageElement.width, imageElement.height)
  return context.getImageData(0, 0, imageElement.width, imageElement.height)
}

const fileToImageData = async (f: File): Promise<ImageData> => {
  let imageElement = await fileToImageELement(f)
  let imageData = imageElementToImageData(imageElement)
  if (!imageData) {
    throw new Error('get imageData failed')
  }
  return imageData
}

const initImageElement = async (src: string): Promise<HTMLImageElement> => {
  return new Promise((resolve, reject) => {
    let image = new Image()
    image.onload = () => {
      resolve(image)
    }
    image.src = src
  })
}

type CanvasRect = {
  width: number
  height: number
}

export default (): JSX.Element => {
  const [images, setImages] = useState<Array<ImageData>>([])
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
    orb.feature_point_matching(55)
    // let matching = Slam.feature_point_matching(images[0], images[1], 80)
    console.log(orb.get_feature_points_1())
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
    }
    let images = []
    for (let i = 0; i < files.length; i++) {
      let imageData = await fileToImageData(files[0])
      images.push(imageData)
    }
    console.log(images)
    setImages(images)
  }

  const drawMathingResult = (
    features1: Uint32Array,
    feautres2: Uint32Array,
    matched: Uint32Array
  ) => {
    console.log(`matched`, matched)
    let image1Width = images[0].width
    let image1Height = images[0].height
    let image2Width = images[0].width
    let image2Height = images[0].height
    if (image1Width != image2Width || image1Height != image2Height) {
      //TODO: padding
    }
  }
  return (
    <div>
      <input
        type='file'
        accept='image/*'
        multiple
        onChange={handleUploadImage}
      />
      <button>特征点匹配</button>
    </div>
  )
}
