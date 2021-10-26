export const fileToImageELement = (file: File): Promise<HTMLImageElement> => {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => {
      const image = new Image()
      image.onload = () => {
        resolve(image)
      }
      image.src = reader.result as string
    }
    reader.readAsDataURL(file)
  })
}

export const imageElementToImageData = (imageElement: HTMLImageElement) => {
  const canvas = document.createElement('canvas')
  canvas.width = imageElement.width
  canvas.height = imageElement.height
  const context = canvas.getContext('2d')
  if (!context) return
  context.drawImage(imageElement, 0, 0, imageElement.width, imageElement.height)
  return context.getImageData(0, 0, imageElement.width, imageElement.height)
}

export const fileToImageData = async (f: File): Promise<ImageData> => {
  let imageElement = await fileToImageELement(f)
  let imageData = imageElementToImageData(imageElement)
  if (!imageData) {
    throw new Error('get imageData failed')
  }
  return imageData
}

export const initImageElement = async (
  src: string
): Promise<HTMLImageElement> => {
  return new Promise((resolve, reject) => {
    let image = new Image()
    image.onload = () => {
      resolve(image)
    }
    image.src = src
  })
}
