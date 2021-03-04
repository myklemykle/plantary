import 'regenerator-runtime/runtime'
import React from 'react'
import ReactDOM from 'react-dom'
import Modal from 'react-bootstrap/Modal'
import Button from 'react-bootstrap/Button'
import { connect, Contract, keyStores } from 'near-api-js'
import { login, logout, vtypes, vnames, ptypes, pnames, initContract } from './utils'
import { AccountOrWallet, WalletLink } from './walletComponents'
import getConfig from './config'

const nearConfig = getConfig(process.env.NODE_ENV || 'development')
const arweaveHost = 'testnet.arweave.net'; // testnet

class SeedTable extends React.Component {
  constructor(props) {
    super(props);
    this.state = {
			seeds: []
			// test:
			/*
			seeds : [ {
				sid: 666,
				name: 'arty',
				vtype: 1,
				vsubtype: 2,
				artist: 'painter of light'
			}]
			*/
		};
	}

	componentDidMount() {
    this.getSeeds();
  }

	getSeedMeta(seed) {
		var sidx = this.state.seeds.findIndex(s => {return s.sid === seed.sid});
		// TODO throw exception if no index.
		var seedMeta = $.getJSON(seed.meta_url)
			.then(s => {
				// munge ...
				Object.assign(this.state.seeds[sidx], s);
				var attrs = {};
				s.attributes.forEach((v,i) => {
					if (v.trait_type === "rarity") return; // don't do this one.
					attrs[v.trait_type] = v.value;
				});
				Object.assign(this.state.seeds[sidx], attrs);
				this.setState({seeds: this.state.seeds});
			})
			// TODO: handle err
	}

	getSeeds() {
		let account = window.walletConnection.account();
    if (window.walletConnection.isSignedIn()) {
			window.contract.get_seeds_page( {page_size:0, page: 0} )
      .then(seeds => {
        this.setState({seeds: seeds});
				seeds.forEach(s => this.getSeedMeta(s) )
      })
    }
      // TODO: handle err
	}

	render() {
		var rows = [];
		this.state.seeds.forEach(function(s,i) {
			rows.push(
				<tr>
					<th scope="row">{s.sid}</th>
					<td><img src={s.image} style={{height: "50px"}} class="seed-image" /></td>
					<td>{s.name}</td>
					<td>{vnames.en[s.vtype] }</td>
					<td>{pnames.en[s.vsubtype ]}</td>
					<td>{s.artist}</td>
					<td>{s.rarity}</td>
					<td><a href={s.meta_url}>{s.meta_url}</a></td>
				</tr>
			)
		});
		
		return (
			<table class="table">
				<thead>
					<tr>
						<th scope="col">sid</th>
						<th scope="col">image</th>
						<th scope="col">name</th>
						<th scope="col">type</th>
						<th scope="col">subtype</th>
						<th scope="col">artist</th>
						<th scope="col">rarity</th>
						<th scope="col">meta_url</th>
					</tr>
				</thead>
				<tbody>
					{ rows }
				</tbody>
			</table>
		)
	}

}


// Main page component
class Intake extends React.Component {
	constructor(props){
		super(props);
		this.state = {
			showModal: false,
			//log: ""
			log: [] // array of jsx elements
		};
		this.handleSubmit = this.handleSubmit.bind(this);
		this.closeModal = this.closeModal.bind(this);
	}

	closeModal() {
		this.setState({showModal: false});
		location.reload();
	}

	log(s){
    console.log(s);
		this.state.log.push( (<p>{s}</p>) );
    this.setState({
			//log: this.state.log + s + "\n",
			log: this.state.log,
      showModal: true
    });
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

	// convert jq form value array [{name: foo, val: bar}, {name:lux, val:breem}] to plan obj {foo:bar, lux:breem}
	//
	objectifyForm(formArray) {
		var returnObj = {};
		for (var i = 0; i < formArray.length; i++){
				returnObj[formArray[i]['name']] = formArray[i]['value'];
		}

		return returnObj;
	}

	handleSubmit(e) {

		e.preventDefault();

		// TODO: add form verification:
		// * near wallet is connected?
		// * user can admin seeds on Plantary?
		// * arweave key is legit?
		
		var imageFile = this.upload.cachedFileArray[0];
		var reader = new FileReader();
		var arKey = JSON.parse($('#arkey').val()); 
		var formObj = this.objectifyForm( $(e.target.form).serializeArray() );
		var account = window.walletConnection.account();

		console.log(formObj);//DEBUG

		// TODO: a simple exception handler around all this, just to display exceptions to screen
		reader.onload = async function() {
			this.log('starting upload ...');

			// Three step process.
			//
			// Step 1: upload/publish image to arweave:
			let transaction1 = await this.arweave.createTransaction({ data: reader.result }, arKey);
			transaction1.addTag('Content-Type', imageFile.type);

			await this.arweave.transactions.sign(transaction1, arKey);

			let arUploader = await this.arweave.transactions.getUploader(transaction1);

			while (!arUploader.isComplete) {
				await arUploader.uploadChunk();
				this.log(`deploying image: ${arUploader.pctComplete}% complete, ${arUploader.uploadedChunks}/${arUploader.totalChunks}`);
			}

			this.log("image deployed.");

			const image_url = this.txToUrl(transaction1.id);
			// Step 2: upload/publish JSON metadata (including URL from step 1)

			// this is our own custom format, somewhat compatible with the mintbase NFT format, 
			// but still different, because this will the metadata for multiple NFTs.
			// So it's not got a 'minter' or 'minted' or 'mintedOn', and also
			// price info & rarity info aren't encoded here.  I hope that someday
			// we'll be able to figure out how to make this easy for marketplaces to use,
			// but the standards simply aren't there today.  For now, this will let us
			// use one decoder for both our new seeds and the existing beta-set of seeds.

			let nftObj = {
				type: "NEP4",
				contractAddress: window.contract.contractId,
				blockchain: "NEAR",
				seeder: account.accountId,
				seeded: "seeded on Plantary",
				seededOn: formObj.created, // check format
				name: formObj.name,
				description: formObj.description,
				image: image_url,
				visibility: formObj.visibility || "safe",
				attributes: [
					{
						trait_type: "vtype",
						value: formObj.vtype
					}, {
						trait_type: "vsubtype",
						value: formObj.vsubtype
					}, {
						trait_type: "artist",
						value: formObj.artist
					} 
				] 
			};

			console.log(nftObj);//DEBUG

			let transaction2 = await this.arweave.createTransaction({ data: JSON.stringify(nftObj) }, arKey);

			transaction2.addTag('Content-Type', 'application/json');

			await this.arweave.transactions.sign(transaction2, arKey);
			console.log("transaction 2:");
			console.log(transaction2);

			arUploader = await this.arweave.transactions.getUploader(transaction2);

			while (!arUploader.isComplete) {
				await arUploader.uploadChunk();
				this.log(`deploying metadata: ${arUploader.pctComplete}% complete, ${arUploader.uploadedChunks}/${arUploader.totalChunks}`);
			}

			const meta_url = this.txToUrl(transaction2.id);
			this.log("metadata deployed, seeding contract ...");

			// Step 3: Create seed record in Plantary contract, including URL from step 2
			//
			var seedid = await window.contract.create_seed({
				vtype: parseInt(formObj.vtype),
				vsubtype: parseInt(formObj.vsubtype),
				meta_url: meta_url,
				rarity: parseInt(formObj.rarity), 
				// edition: parseInt(formObj.edition), // not in form yet
				edition: 1,
			});

			// ... this sometimes redirects to the Near wallet and sometimes not, depending on the cost of the data?
			
			this.log("Seed planted!");
			// TODO: update seed list!
			// tell seed list to update?
			// tell this to re-render seed list?
			// reload page?
			// react ... you so weird

		}.bind(this);

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
												<input type="radio" id="vtype-plant" name="vtype" value="1" class="custom-control-input" />
												<label class="custom-control-label" for="vtype-plant">Plant</label>
											</div>
											<div class="custom-control custom-radio custom-control-inline">
												<input type="radio" id="vtype-harvest" name="vtype" value="2" class="custom-control-input" />
												<label class="custom-control-label" for="vtype-harvest">Harvest</label>
											</div>
										</div>
									</div>
									<div class="form-group row">
										<label for="vsubtype" class="col-3 col-form-label">Type</label> 
										<div class="col-9">
											<select class="form-control" id="vsubtype" name="vsubtype">
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
											<input type="range" class="form-range" min="1" max="10" step="0.5" name="rarity" id="rarity" />
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
							</div>
						</section>
						<section class="page-section seedtable" id="seedtable">
									<SeedTable />
						</section>
			
			<Modal show={this.state.showModal} onHide={this.closeModal}>
        <Modal.Header closeButton>
          <Modal.Title>Planting ...</Modal.Title>
        </Modal.Header>
        <Modal.Body>{this.state.log}</Modal.Body>
        <Modal.Footer>
          <Button variant="secondary" onClick={this.closeModal}>
            Close
          </Button>
        </Modal.Footer>
      </Modal>
					

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

