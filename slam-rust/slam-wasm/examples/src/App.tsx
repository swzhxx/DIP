import React from 'react'
import logo from './logo.svg'
import './App.css'
import { HashRouter as Router, Route, Switch } from 'react-router-dom'
import SingleView from '@/views/SingleView'
import FeatureMatching from './views/FeatureMatching'
import Recover from './views/Recover'
function App() {
  return (
    <div className='App'>
      <Router>
        <Switch>
          <Route exact path='/single-view'>
            <SingleView></SingleView>
          </Route>
          <Route exact path='/feature-match'>
            <FeatureMatching></FeatureMatching>
          </Route>
          <Route exact path='/recover'>
            <Recover></Recover>
          </Route>
        </Switch>
      </Router>
    </div>
  )
}

export default App
