import React from 'react'
import { login, logout, mintPlant } from './utils'
import getConfig from './config'

const nearConfig = getConfig(process.env.NODE_ENV || 'development')

export function AccountOrWallet() {
	if (window.walletConnection.isSignedIn()) 
		return window.walletConnection.getAccountId();
	else
		return "CONNECT WALLET";
}

export function MineOrWallet() {
	if (window.walletConnection.isSignedIn()) 
		return "MY PLANTARY";
	else
		return "CONNECT WALLET";
}

export function WalletLink() {
  function handleClick(e) {
    e.preventDefault();
		if (!window.walletConnection.isSignedIn()) {
			login();
		} else {
			logout();
		}
  }

	let faIcon = window.walletConnection.isSignedIn() ? "fa-sign-out-alt" : "fa-cog";

  return (
		<a href="#" className="btn btn-outline-light btn-social mx-1" onClick={handleClick} ><i className={'fas ' + faIcon}></i></a>
  );
}

export function MintPlantButton(props){
  function handleClick(e) {
    e.preventDefault();
		if (window.walletConnection.isSignedIn()) {
			mintPlant(props.pType, props.price);
		} else {
			login();
		}
	}

	return (
		<button className="btn btn-primary" href="#" onClick={handleClick} data-dismiss="modal"><i className="fas fa-seedling"></i> Mint Plant</button>
	)
}

