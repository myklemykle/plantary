#!/bin/zsh

. ./.env || exit 1 # project env

declare -A vtypes
vtypes[PLANT]=1
vtypes[HARVEST]=2

declare -A vcats
vcats[ORACLE]=1
vcats[PORTRAIT]=2
vcats[MONEY]=3
vcats[COMPLIMENT]=4
vcats[INSULT]=5
vcats[SEED]=6


        # type, subtype, meta_url, rarity, edition

#create_seed vtype, vcat, meta_url, rarity, edition){
create_seed(){
  npx near call --accountId $ADMIN_ID $CONTRACT_NAME 'create_seed' "{\"vtype\": $1, \"vcat\": $2, \"meta_url\": \"$3\", \"rarity\":$4, \"edition\": $5}" 
}


        create_seed $vtypes[PLANT] $vcats[ORACLE] \
    "https://3bvdryfdm3sswevmvr3poka2ucda5dfqag3bz4td72affctbmaea.arweave.net/2Go44KNm5SsSrKx29ygaoIYOjLABthzyY_6AUophYAg"\
            5.0 1
        
        create_seed $vtypes[PLANT] $vcats[ORACLE] \
    "https://vwanp7rn32rioq6ofcvglo52sgdrctcfkc4v7uiy7bbimtzijz3q.arweave.net/rYDX_i3eoodDziiqZbu6kYcRTEVQuV_RGPhChk8oTnc"\
            5.0 1
        
        create_seed $vtypes[PLANT] $vcats[ORACLE] \
    "https://arweave.net/VoJ1Wx6xTflalopLxOuj7TpO8pC0urYB-vLiZ1FxYno"\
            5.0 1
        
        create_seed $vtypes[PLANT] $vcats[ORACLE] \
    "https://arweave.net/33wa-6hW_vQAbkQ4a5ZXX7HGJMGR3M8ej-z9dvcnJ8k"\
            5.0 1
        


        create_seed $vtypes[PLANT] $vcats[PORTRAIT] \
    "https://rsigfpny3j3uwohxfeo7tdkdvw6yhaefxt6d3uq7kajtpaqtdfwq.arweave.net/jJBivbjad0s49ykd-Y1Drb2DgIW8_D3SH1ATN4ITGW0"\
            5.0 1
        
        create_seed $vtypes[PLANT] $vcats[PORTRAIT] \
    "https://arweave.net/fo--Wlh83Ka83zVQqliiwFq_4zbc1H7vrZNlvA_Gkek"\
            5.0 1
        
        create_seed $vtypes[PLANT] $vcats[PORTRAIT] \
    "https://arweave.net/1oDuE6UNrNC4Y_aNfhp_Vde_II2ZIFsuRT1hBYbRydc"\
            5.0 1
        
        create_seed $vtypes[PLANT] $vcats[PORTRAIT] \
    "https://arweave.net/M7uwpTyRIZIohXBgIZUqoYDyxq1GyH3fkoT7CvN2iLE"\
            5.0 1
        

        create_seed $vtypes[PLANT] $vcats[MONEY] \
    "https://rj32ukhcq4hdq7nux3rntp5ffdk3ff2kzjcalpy3mc7batjytoza.arweave.net/ineqKOKHDjh9tL7i2b-lKNWyl0rKRAW_G2C-EE04m7I"\
            5.0 1
        
        create_seed $vtypes[PLANT] $vcats[MONEY] \
    "https://b2zjlf2zplj5we2bdar6p6smu3o6fdu7o7ed23takt63lck6peoq.arweave.net/DrKVl1l609sTQRgj5_pMpt3ijp93yD1uYFT9tYleeR0"\
            5.0 1
        
        create_seed $vtypes[PLANT] $vcats[MONEY] \
    "https://arweave.net/q8RPmg2qf6nfE4Gc1at7bqOBuWbSsEtzxvdICb1NYzk"\
            5.0 1
        
        create_seed $vtypes[PLANT] $vcats[MONEY] \
    "https://arweave.net/dPBN2DGba13xI7IFqBoczspsHbTXUvmZ9sjKIuhU28o"\
            5.0 1



      create_seed $vtypes[HARVEST] $vcats[ORACLE] \
  "https://arweave.net/v63RbTVHhGKr7UNMmwMjBtKepk1I26UB4yxPhJVSkcg"\
          5.0 1
      
      create_seed $vtypes[HARVEST] $vcats[ORACLE] \
  "https://arweave.net/hvOKZAw3miEA8BE4VewzH9io4fNsSWyZpGZaSmhr-l8"\
          5.0 1
      
      create_seed $vtypes[HARVEST] $vcats[ORACLE] \
  "https://arweave.net/B_c8uZaUFIA8hjLDVr3v4IR6aRT-zzvCaE0cqWgVURc"\
          5.0 1
      
      create_seed $vtypes[HARVEST] $vcats[ORACLE] \
  "https://arweave.net/mGhn0lNxVB6rfon61c9rRioMKL3ZsbjrVJA0qt9St4o"\
          5.0 1
      
      create_seed $vtypes[HARVEST] $vcats[ORACLE] \
  "https://arweave.net/3xMnn8J1ViLX8uHRfDxMpAZS2tSwT7VWrdjDT3fV2xQ"\
          5.0 1
      
      create_seed $vtypes[HARVEST] $vcats[ORACLE] \
  "https://arweave.net/_q0UfS76GMma9PR-XMavRI8ozipY_cmgoi6TFS_eHOg"\
          5.0 1
      
      create_seed $vtypes[HARVEST] $vcats[ORACLE] \
  "https://arweave.net/-Nxk3noWBskl8kbfxhCZFspD7v9lf79iJt1bQ2TCTzw"\
          5.0 1
      
      create_seed $vtypes[HARVEST] $vcats[ORACLE] \
  "https://arweave.net/zXIyOvf6q42eiVnixg6EK_RmFfxlZaFuaQgvs9b6Y8c"\
          5.0 1
      
      create_seed $vtypes[HARVEST] $vcats[ORACLE] \
  "https://arweave.net/u2wER7li2oXgMXfRUs22oERc-XUsn2Ph9yBsZPrvcBc"\
          5.0 1
      
      create_seed $vtypes[HARVEST] $vcats[ORACLE] \
  "https://arweave.net/eQQKfobStzP8dHIzbXYjJCMKQR1owIZ5ljjwX3xvz7I"\
          5.0 1
      
			 	# Oracle Voucher, by Oculardelusion
      create_seed $vtypes[HARVEST] $vcats[ORACLE] \
  "https://arweave.net/eYJ3Ie8K3sVwvXSH5xtXtyi8PizJCzkXK4n7MCCMunE"\
          5.0 1
      

      create_seed $vtypes[HARVEST] $vcats[PORTRAIT] \
  "https://arweave.net/tmOUL9xwL8LQb_E5kOldLaF0mrZLg9rSMYpoTGgdkU8"\
          5.0 1
      
      create_seed $vtypes[HARVEST] $vcats[PORTRAIT] \
  "https://arweave.net/tvCQax-rq-oDvRdy-QnBp5orrjSP04Y-dNxXC3maTkI"\
          5.0 1
      
      create_seed $vtypes[HARVEST] $vcats[PORTRAIT] \
  "https://arweave.net/CJyoNeeDM_Vco0l4-7y434_pe4hBhWEE9vvh5XqMd4k"\
          5.0 1
      
      create_seed $vtypes[HARVEST] $vcats[PORTRAIT] \
  "https://arweave.net/hZ3etzVzsaXX6utSldyfvIvp0JUoFISuA72vJpNKa8s"\
          5.0 1
      
      create_seed $vtypes[HARVEST] $vcats[PORTRAIT] \
  "https://arweave.net/63vqengRMJiiBU-YmVpRH9nDclZB_f3zNVsn0wtcqg4"\
          5.0 1
      
      create_seed $vtypes[HARVEST] $vcats[PORTRAIT] \
  "https://arweave.net/mTFjapdnAWqWLXOqySLM0cHX2AOcYYWpoSPArBA_suk"\
          5.0 1
      
      create_seed $vtypes[HARVEST] $vcats[PORTRAIT] \
  "https://arweave.net/eaCk3l8Oi3MqNi7lKMRRC7gR5zRXO9JfbJc80OquHQk"\
          5.0 1
      
      create_seed $vtypes[HARVEST] $vcats[PORTRAIT] \
  "https://arweave.net/30kVPubXOw6vJce923j6Nv27jWl39AeS4EcpMijCmZA"\
          5.0 1
      
      create_seed $vtypes[HARVEST] $vcats[PORTRAIT] \
  "https://arweave.net/1xw0bRDaU-CV7hsOGnP51ZWr5_zVk21Qxu8h_jcX-tg"\
          5.0 1

      create_seed $vtypes[HARVEST] $vcats[PORTRAIT] \
  "https://arweave.net/FK4nE9euzIoEx4QOPpocSKDK0wjrPwcxX0cjxx8Km5I"\
          5.0 1
      
      create_seed $vtypes[HARVEST] $vcats[PORTRAIT] \
  "https://arweave.net/zIU6uG94XnwtTeEzHB3GSikBTtErmE3fgWCZV744tZE"\
          5.0 1
      
      create_seed $vtypes[HARVEST] $vcats[PORTRAIT] \
  "https://arweave.net/usIqVRzLyFNGENUgeV8c5-zjzEptOJZa23BkUDiU3cU"\
          5.0 1
      
      create_seed $vtypes[HARVEST] $vcats[PORTRAIT] \
  "https://arweave.net/EDiBwvIYUmT5cmPqbW02HOuFZnHUPoTNX_ri3N2BeTg"\
          5.0 1
      
				# Dennis, by Ilan Katin
      create_seed $vtypes[HARVEST] $vcats[PORTRAIT] \
  "https://arweave.net/LJkU3DnETelIpCdn6l6v-ZDUdY0LzW77G3qNPkRl7cs"  \
          5.0 1
      
				# Portrait Voucher, by Oculardelusion
      create_seed $vtypes[HARVEST] $vcats[PORTRAIT] \
  "https://arweave.net/MiV3Xi4qmRjquQJTK3usefWMy62DJ1TCemb0jsY1NBo"  \
          5.0 1

echo "done"
