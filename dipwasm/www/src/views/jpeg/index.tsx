import React, { useRef } from 'react'
import CanvasWithImg from '@/components/CanvasWithImg'
import { Button } from 'antd'
import { Splat } from '@/wasm'
export default (): JSX.Element => {
  const cRef = useRef<any>(null)
  const onClick = () => {
    let imageData = cRef.current.getOriginData()
    imageData = Splat.splatJpeg(
      imageData.data,
      imageData.width,
      imageData.height
    )
    cRef.current.putImageData(imageData)
  }

  return (
    <div>
      <CanvasWithImg
        ref={cRef}
        canvas={{
          width: '512px',
          height: '512px',
        }}
        src={`${process.env.PUBLIC_URL}/images/lena_512.jpg`}
      ></CanvasWithImg>
      <Button onClick={onClick}>Jpeg</Button>
    </div>
  )
}
