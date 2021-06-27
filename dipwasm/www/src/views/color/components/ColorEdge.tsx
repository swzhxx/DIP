import CanvasWithImg from '@/components/CanvasWithImg'
import { Button } from 'antd'
import React, { ReactInstance, useRef } from 'react'
import { Splat } from '@/wasm'
export default (): JSX.Element => {
  const cRef = useRef<any>(null)

  const onClick = () => {
    let imageData = cRef.current.getOriginData()
    imageData = Splat.splatColorEdge(
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
          width: '256px',
          height: '256px',
        }}
        src={`${process.env.PUBLIC_URL}/images/lena_256.jpg`}
      ></CanvasWithImg>
      <Button onClick={onClick}>边界</Button>
    </div>
  )
}
