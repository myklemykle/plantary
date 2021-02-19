import 'regenerator-runtime/runtime'
import React from 'react'
import { login, logout, mintPlant } from './utils'
import { Home } from './Home'

import getConfig from './config'
//const { networkId } = getConfig(process.env.NODE_ENV || 'development') // borked?
const { networkId } = getConfig('testnet');

export default function App() {
	return (
		<Home />
	)
}
