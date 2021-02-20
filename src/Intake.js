import 'regenerator-runtime/runtime'
import React from 'react'
import ReactDOM from 'react-dom'
import { connect, Contract, keyStores, WalletConnection } from 'near-api-js'
import { login, logout, ptypes, initContract } from './utils'
import { AccountOrWallet, WalletLink } from './walletComponents'
import getConfig from './config'

const nearConfig = getConfig(process.env.NODE_ENV || 'development')
const arweaveHost = 'testnet.arweave.net'; // testnet

// Component: progress block
class ProgressBlock extends React.Component {
	constructor(props) {
		super(props);
		props = props || {};
		this.state = { 
			log: props.log || "" ,
			display: props.display || "none"
		};
	}

	log(s){
		console.log(s);
		this.setState({
			log: this.state.log + s +  "<br\>",
			display: "inline"
		}); 
		// maybe React doesn't render during event handlers?  which is the only place we use this?
		// Anyway, forcing the issue:
		this.forceUpdate();
	}

	clear(){
		this.setState({
			log: "",
			display: "none"
		})
	}

	render() { 
		return (
			<div id="progressblock" class="row" style={{display: this.state.display}}>
				<div class="col-3 col-form-label">Deployment progress:</div>
				<div id="progress" class="col-9">
					{this.state.log}
				</div>
			</div>
		)
	}
}

const progress = new ProgressBlock();

// reset entire form
function resetForm(e){
	e.preventDefault();
	let theForm = $('#sow');
	theForm.find('input,textarea:not([name="arkey"])').val('');
	progress.clear();
}

// Component: reset button
class ResetButton extends React.Component {
  constructor(props) {
    super(props);
    this.state = {
			display: "none"
		};
	}

	resetClick() {
		resetForm();
		this.hide();
	}

	hide(){ 
		this.setState({display: "none"});
	}
	show(){ 
		this.setState({display: "inline"});
	}

		// utility: clear the whole form except the key

	render() { 
		return (
			<div id="formResetButton" class="form-group row" style={{display: this.state.display}}>
				<div class="offset-3 col-9">
					<button name="reset" type="button" onClick={resetForm} class="btn btn-primary">Next!</button>
				</div>
			</div>
		)
	}
}

// Main page component
class Intake extends React.Component {
	constructor(props){
		super(props);
		this.state = {
		};
		this.handleSubmit = this.handleSubmit.bind(this);
	}

	// attach libraries to DOM after render
	componentDidMount() {
		// jq find stuff:
		this.theForm = $('#sow');

		// init file uploader
		this.upload = new FileUploadWithPreview("myUniqueUploadId", {
				showDeleteButtonOnImages: true,
		});

		// init arweave
		this.arweave = window.Arweave.init({
			host: arweaveHost, 
		});

		/* 
		 *
		 * plugin doesn't play well with webpack, i think cuz it attaches to jquery via Window ...
		*/
		// init datepicker
		//var created = this.theForm.find('#created');
		var created = $('#created');
		created.datepicker();
		// if date is blank, set to today
		// (are there reload situations where the previous value would still be here?)
		if (created.val().trim() === "") {
			created.datepicker('setDate', new Date());
		}

	}

		// utility: return link to URL for our tx
	txToUrl(txid) {
		return "https://" + arweaveHost + '/' + txid
	}
	txToLink(txid) {
		let url = this.txToUrl(txid);
		return '<a href="' + url + '" target="_blank">' + url + '</a>';
	}

		// utility: convert form to json
	objectifyForm(formArray) {
		//serialize data function
		var returnArray = {};
		for (var i = 0; i < formArray.length; i++){
				returnArray[formArray[i]['name']] = formArray[i]['value'];
		}
		return returnArray;
	}

	handleSubmit(e) {

		e.preventDefault();

		// TODO: verify some stuff:
		// * near wallet is connected
		// * near user can admin seeds on Plantary
		// * arweave key is legit
		
		var imageFile = this.upload.cachedFileArray[0];
		var reader = new FileReader();
		var key = JSON.parse($('#arkey').val()); 
		var formJSON = this.objectifyForm($(e.target.form).serializeArray());
		console.log(formJSON);//DEBUG

		// TODO: a simple exception handler around all this, just to display exceptions to screen
		reader.onload = async function() {

			// Three step process.
			//
			// Step 1: upload/publish image to arweave:
			let transaction1 = await this.arweave.createTransaction({ data: reader.result }, key);
			transaction1.addTag('Content-Type', imageFile.type);

			await this.arweave.transactions.sign(transaction1, key);

			let arUploader = await this.arweave.transactions.getUploader(transaction1);

			while (!arUploader.isComplete) {
				await arUploader.uploadChunk();
				progress.log(`deploying image: ${arUploader.pctComplete}% complete, ${arUploader.uploadedChunks}/${arUploader.totalChunks}`);
			}

			progress.log("image deployed: " + this.txToLink(transaction1.id));

			// Step 2: upload/publish JSON metadata (including URL from step 1)
			let nftObj = formJSON;
			delete formJSON.arkey;
			nftObj.image = this.txToUrl(transaction1.id);
			nftObj.visibility = nftObj.visibility || "safe";
			delete nftObj.MAX_FILE_SIZE;
			let transaction2 = await this.arweave.createTransaction({ data: JSON.stringify(nftObj) }, key);

			transaction2.addTag('Content-Type', 'application/json');

			await this.arweave.transactions.sign(transaction2, key);
			console.log("transaction 2:");
			console.log(transaction2);

			arUploader = await this.arweave.transactions.getUploader(transaction2);

			while (!arUploader.isComplete) {
				await arUploader.uploadChunk();
				progress.log(`deploying metadata: ${arUploader.pctComplete}% complete, ${arUploader.uploadedChunks}/${arUploader.totalChunks}`);
			}
			const meta_url = this.txToLink(transaction2.id);
			progress.log("metadata deployed: " + meta_url);

			// Step 3: Create seed record in Plantary contract, including URL from step 2
			//
			var seedid = await window.contract.create_seed({
				vtype: nftObj.vtype,
				vsubtype: nftObj.vsubtype,
				meta_url: meta_url,
				rarity: nftObj.rarity, 
				// edition: nftObj.edition,
				edition: 1,
			});

			progress.log("seed " + seedid + "planted");

			progress.log("done!");
			resetButton.show();

		}.bind(this);

		// maybe this buffer already exists in upload, but i can't find it ...
		reader.readAsArrayBuffer(imageFile); // triggers reader.load ...
	}


	// Return main page with form
	render(){
		return (
			<>
					<nav class="navbar navbar-expand-lg bg-secondary fixed-top" id="mainNav">
							<div class="container"><a class="navbar-brand js-scroll-trigger" href="#page-top">PLANTARY</a>
									<button class="navbar-toggler navbar-toggler-right font-weight-bold bg-primary text-white rounded" type="button" data-toggle="collapse" data-target="#navbarResponsive" aria-controls="navbarResponsive" aria-expanded="false" aria-label="Toggle navigation">Menu <i class="fas fa-bars"></i></button>
									<div class="collapse navbar-collapse" id="navbarResponsive">
											<ul class="navbar-nav ml-auto">
											</ul>
									</div>
							</div>
					</nav>
					<header class="masthead bg-primary text-white text-center">
							<div class="container d-flex align-items-center flex-column">
								{/* Masthead Heading */}
									<h1 class="masthead-heading mb-0">Sow Your Art to Reap NFTs</h1>
									{/* Icon Divider */}
									<div class="divider-custom divider-light">
											<div class="divider-custom-line"></div>
											<div class="divider-custom-icon"><i class="fas fa-star"></i></div>
											<div class="divider-custom-line"></div>
									</div>
									{/* Masthead Subheading */}
									<p class="pre-wrap masthead-subheading font-weight-light mb-0">Seed your digital art in the Plantary</p>
							</div>
					</header>
					<section class="page-section portfolio" id="portfolio">
						<div class="container">
									{/* Main Form */}
							<form id="sow">
									<div class="form-group row">
										<label for="vtype" class="col-3 col-form-label">Plant or Harvest?</label> 
										<div class="col-9">
											<div class="custom-control custom-radio custom-control-inline">
												<input type="radio" id="vtype-plant" name="vtype" class="custom-control-input" />
												<label class="custom-control-label" for="vtype-plant">Plant</label>
											</div>
											<div class="custom-control custom-radio custom-control-inline">
												<input type="radio" id="vtype-harvest" name="vtype" class="custom-control-input" />
												<label class="custom-control-label" for="vtype-harvest">Harvest</label>
											</div>
										</div>
									</div>
									<div class="form-group row">
										<label for="vsubtype" class="col-3 col-form-label">Type</label> 
										<div class="col-9">
											<select class="form-control" id="type">
												<option>Choose ...</option>
												<option value="1">Oracle</option>
												<option value="2">Portrait</option>
												<option value="3">Money</option>
												{/*
												<option value="4">Compliment</option>
												<option value="5">Insult</option>
												<option value="6">Seed</option>
												 */}
											</select>
										</div>
									</div>
									<div class="form-group row">
										<label for="rarity" class="col-3 col-form-label">Rarity</label> 
										<label for="rarity" class="col-2">Omnipresent</label>
										<div class="col-5 range">
											<input type="range" class="form-range" min="1" max="10" step="0.5" id="rarity" />
										</div>
										<label for="rarity" class="col-2">Nonexistent</label>
									</div>
									{/* file upload with preview  */}
									<div class="custom-file-container" data-upload-id="myUniqueUploadId">
										<div class="form-group row">
											<label for="image2" class="col-3 col-form-label custom-file-container__image-clear">Image (350x350px)</label>
											<div class="col-9">
												<div class="input-group">
													<div class="input-group-prepend">
														<div class="input-group-text">
															<i class="fa fa-image"></i>
														</div>
													</div> 
													<div class="custom-file">
															<label class="custom-file-label" for="image2">
																	<input
																			id="image2"
																			type="file"
																			class="custom-file-input custom-file-container__custom-file__custom-file-input"
																			accept="*"
																			required="required"
																			aria-label="Choose File"
																	/>
																	<input type="hidden" name="MAX_FILE_SIZE" value="10485760" />
																	<span
																			class="custom-file-container__custom-file__custom-file-control cfu-tweaks" 
																	></span>
															</label>
													</div>
												</div>
											</div>
										</div>
										<div class="form-group row">
											<div class="col-3"></div>
											<div class="col-9">
														<div class="custom-file-container__image-preview"></div>
											</div>
										</div>
									</div>
									<div class="form-group row">
										<label for="name" class="col-3 col-form-label">Title</label> 
										<div class="col-9">
											<div class="input-group">
												<div class="input-group-prepend">
													<div class="input-group-text">
														<i class="fa fa-tag"></i>
													</div>
												</div> 
												<input id="name" name="name" type="text" aria-describedby="nameHelpBlock" required="required" class="form-control" />
											</div> 
										</div>
									</div>
									<div class="form-group row">
										<label for="artist" class="col-3 col-form-label">Artist</label> 
										<div class="col-9">
											<div class="input-group">
												<div class="input-group-prepend">
													<div class="input-group-text">
														<i class="fa fa-paint-brush"></i>
													</div>
												</div> 
												<input id="artist" name="artist" type="text" class="form-control" />
											</div>
										</div>
									</div>
									<div class="form-group row">
										<label for="description" class="col-3 col-form-label">Description</label> 
										<div class="col-9">
											<textarea id="description" name="description" cols="40" rows="5" class="form-control" required="required"></textarea>
										</div>
									</div>
									<div class="form-group row">
										<label for="created" class="col-3 col-form-label">Created on</label> 
										<div class="col-9">
											<div class="input-group">
												<div class="input-group-prepend">
													<div class="input-group-text">
														<i class="fa fa-calendar"></i>
													</div>
												</div> 
												<input id="created" name="created" type="text" class="form-control" 
												data-provide="datepicker" 
												required="required" />
											</div>
										</div>
									</div>
									{/*
									<div class="form-group row">
										<label class="col-3">Adult content?</label> 
										<div class="col-9">
											<div class="form-check form-check-inline">
												<input name="visibility" id="visibility_0" type="checkbox" class="form-check-input" value="unsafe" /> 
												<label for="visibility_0" class="form-check-label">NSFW</label>
											</div>
										</div>
									</div> 
									 */}
									<div class="form-group row">
										<label for="arkey" class="col-3 col-form-label">Arweave wallet JSON</label> 
										<div class="col-9">
											<textarea id="arkey" name="arkey" cols="40" rows="3" class="form-control" required="required"></textarea>
										</div>
									</div>
									<div class="form-group row">
										<label for="nearWallet" class="col-3 col-form-label">NEAR login</label> 
										<div class="col-9">
											<AccountOrWallet /><WalletLink />
										</div>
									</div>
									<div class="form-group row">
										<div class="offset-3 col-9">
											<button name="submit" type="submit" onClick={this.handleSubmit} class="btn btn-primary">Submit</button>
										</div>
									</div>
	</form>
									<ProgressBlock />
								  <ResetButton />
							</div>
						</section>
					

						{/* Copyright Section */}
						<section class="copyright py-4 text-center text-white">
								<div class="container"><small class="pre-wrap">Copyright © Plantary 2021</small></div>
						</section>
						{/* Scroll to Top Button (Only visible on small and extra-small screen sizes) */}
						<div class="scroll-to-top d-lg-none position-fixed"><a class="js-scroll-trigger d-block text-center text-white rounded" href="#page-top"><i class="fa fa-chevron-up"></i></a></div>

				</>
		)
	}
}

window.nearInitPromise = initContract()
  .then(() => {
		$( document ).ready(() => {
			ReactDOM.render(
				<Intake />,
				document.querySelector('#page-top')
			)
		})
	})
	.catch(console.error)

