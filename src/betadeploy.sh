##  beta site redeployment steps:

## i find that nmp as a build tool gets this all tied in knots.

##  -- first, turn off 'yarn run'/parcel server, if it's running, cuz it scribbles in build.

##  -- set up env
## NEAR_TARGET can be beta, production or development .
	export NEAR_TARGET=beta
	. ./.env
##  -- delete /dist to clean up any parcel mess
	rm -r dist
##  -- run tests, build everything
	yarn test
	yarn build
##  -- delete and recreate the beta Plantary user on Near
	npx near delete $CONTRACT_NAME $ADMIN_ID # this should break the beta site.
	npx near create-account $CONTRACT_NAME --masterAccount $ADMIN_ID 
##  -- push dist files to webserver
	yarn push:beta 	# which also triggers a rebuild now?  what a mess ... package.json == spaghetti
##  -- deploy the beta contract
	npx near deploy $CONTRACT_NAME
##  -- initialize the contract
	npx near call --accountId $ADMIN_ID $CONTRACT_NAME new '{"owner_id": "'$ADMIN_ID'"}'
##  -- load the seeds in the contract
	src/load_default_seeds.sh
##  -- test a mint and a harvest
##  -- check on seeds
##  -- lunch
