import React from 'react'
import { Tabs } from 'antd'
import GaussianFilter from './components/GaussianFilter'
import ColorEdge from './components/ColorEdge'
const { TabPane } = Tabs

export default (): JSX.Element => {
  return (
    <Tabs>
      <TabPane tab='高斯模糊' key={1}>
        <GaussianFilter></GaussianFilter>
      </TabPane>
      <TabPane tab='颜色边缘检测' key={2}>
        <ColorEdge></ColorEdge>
      </TabPane>
    </Tabs>
  )
}
