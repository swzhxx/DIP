import CanvasWithImg from '@/components/CanvasWithImg'
import DatGui, { DatNumber, DatSelect } from 'react-dat-gui'
import React, { useState, useRef, useCallback } from 'react'
import { wasm } from '@/wasm'
import _ from 'lodash'
export default (props: any): JSX.Element => {
  const [state, setState] = useState<any>({
    kernel: '1',
  })

  const cRef = useRef<any>(null)

  const debounceHandleFilter = useCallback(
    _.debounce((kernel) => {
      let imageData = cRef.current.getOriginData()
      // imageData = wasm.splatGaussianFilter(imageData, Number(state.kernel))
      // let imageData = new ImageData(
      //   new Uint8ClampedArray(512 * 512 * 4).fill(0),
      //   512,
      //   512
      // )
      if (kernel != 1) {
        imageData = wasm.splatGaussianFilter(
          imageData.data,
          imageData.width,
          imageData.height,
          Number(kernel)
        )
      }

      cRef.current.putImageData(imageData)
    }, 200),
    [state.kernel]
  )

  const handleUpdateState = (newData: any) => {
    let { kernel } = newData
    if (kernel % 2 == 0) {
      kernel = kernel - 1
    }
    console.log({ kernel })
    if (kernel !== state.kernel) {
      setState({ ...newData, kernel })
      if (cRef.current == null) return
      // handleFilter()
      // setTimeout(handleFilter, 200)
      debounceHandleFilter(kernel)
    }
  }

  return (
    <div>
      <DatGui data={state} onUpdate={handleUpdateState}>
        <DatSelect
          options={[1, 3, 5, 7, 9]}
          path='kernel'
          label='ConvWindowSize'
        ></DatSelect>
      </DatGui>
      <CanvasWithImg
        src={`${process.env.PUBLIC_URL}/images/lena_256.jpg`}
        ref={cRef}
        canvas={{
          width: '256px',
          height: '256px',
        }}
      ></CanvasWithImg>
    </div>
  )
}
