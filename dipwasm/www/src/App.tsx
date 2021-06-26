import React from 'react'
import './App.css'
import { HashRouter as Router, Route, Switch, Link } from 'react-router-dom'
import HelloWasm from '@/views/hello-wasm/index'
import Color from '@/views/color/index'
import 'react-dat-gui/dist/index.css'
function App() {
  return (
    <div className='App'>
      <Router>
        <div>
          <ul>
            <Link to='/hello-wasm'>Hello Wasm</Link>
            <Link style={{ marginLeft: '5px' }} to='/color'>
              Color
            </Link>
          </ul>
        </div>
        <div style={{ height: '1px', color: '#999' }}></div>
        <Switch>
          <Route exact path='/hello-wasm'>
            <HelloWasm></HelloWasm>
          </Route>
          <Route exact path='/color'>
            <Color></Color>
          </Route>
        </Switch>
      </Router>
    </div>
  )
}

export default App
