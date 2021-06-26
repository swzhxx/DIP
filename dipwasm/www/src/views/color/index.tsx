import React from 'react'
import { Tabs } from 'antd'
import GaussianFilter from './components/GaussianFilter'
const { TabPane } = Tabs

export default (): JSX.Element => {
  return (
    <Tabs>
      <TabPane tab='高斯模糊'>
        <GaussianFilter></GaussianFilter>
      </TabPane>
      <TabPane></TabPane>
      <TabPane></TabPane>
    </Tabs>
  )
}
