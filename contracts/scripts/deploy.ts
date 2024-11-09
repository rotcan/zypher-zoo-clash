// We require the Hardhat Runtime Environment explicitly here. This is optional
// but useful for running the script in a standalone fashion through `node <script>`.
//
// You can also run a script with `npx hardhat run <script>`. If you do that, Hardhat
// will compile your contracts, add the Hardhat Runtime Environment's members to the
// global scope, and execute the script.

import { ContractTransactionResponse } from "ethers";
import { IState } from "../typechain-types";
import { ErrorDecoder } from 'ethers-decode-error'
import { libraries } from "../typechain-types/contracts";

const errorDecoder = ErrorDecoder.create()
//Current addresses
const LibraryAddress="0x9ecF20487e94B3832AD5284Ff15FaF278341aB4f";//"0x03D73a99f2151DC1b0969bdf436e6dF5f55Ec855";
const MockRevealVerifierAddress="0x362D393756caAe6ECF434D34d5CD70E83bAD02EF";
const MockShuffleVerifierAddress="0x1aab73Ff73c3e861955ad00c909B27c0AB5626EB";
const GameCardAddress="0x5b0a97935cafad5b174e63d01360dad5303bdf19";//"0x7A9B296Ad4c4c832e99127AAd6D58fD8bF8C46Dd";
const opBNBTestnetVRFAddress="0x2B30C31a17Fe8b5dd397EF66FaFa503760D4eaF0";
const VRFAddress ="0x5ce2AB0ea6D7a300a4b440F6C381A35373a6A4a3";//"0x95b5E4AB677BabcbA12D780a86E5a0373480A35f";
const GameAddress="0xe9893007f1bfcec9d655b33cc500cf0dd6648923";
const ZypherAddress="0xe09f5310419e0d0bc4e72c02e21006f499a362ce";
//const hre = require("hardhat");
const hre =require("hardhat");
type Contract<T> = T & { deploymentTransaction(): ContractTransactionResponse; }; 
const sleep=(ms:number )=>{return new Promise(resolve=>setTimeout(resolve,ms))};
async function deployMockContracts(){
    const deck_num = 20;
    const RevealVerifier=await hre.ethers.getContractFactory('RevealVerifier');
    const MockRevealVerifier=await RevealVerifier.deploy();
    
    const VerifierKeyExtra1_20=await hre.ethers.getContractFactory("VerifierKeyExtra1_20");
    const verifierKeyExtra1_20=await VerifierKeyExtra1_20.deploy();
    
    const VerifierKeyExtra2_20=await hre.ethers.getContractFactory("VerifierKeyExtra2_20");
    const verifierKeyExtra2_20=await VerifierKeyExtra2_20.deploy();
    
    const ShuffleVerifier=await hre.ethers.getContractFactory('ShuffleService');
    const MockShuffleVerifier=await ShuffleVerifier.deploy(await verifierKeyExtra1_20.getAddress(),
        await verifierKeyExtra2_20.getAddress(),deck_num);
    console.log(`MockRevealVerifier address ${(await MockRevealVerifier.getAddress())}`);
    console.log(`MockShuffleVerifier address ${(await MockShuffleVerifier.getAddress())}`);
}

async function deployLibrary(){
    const IState= await hre.ethers.getContractFactory("IState");
    const contract=await IState.deploy();
    console.log(`Library deployed to ${(await contract.getAddress())}`)
}

async function deployCard(){
    const Nft= await hre.ethers.getContractFactory('RaceGameCardA',
        {libraries:{
        IState:LibraryAddress
    }}
    );
    const gameCard = await Nft.deploy();
    //const gameCard=await Nft.attach(GameCardAddress);
    await sleep(30_000);
    const gameAddress=(String) (await gameCard.getAddress());
    console.log(`GameCard deployed to ${(gameAddress.toLowerCase())}`);
    const tx=await gameCard.initialize('ZooClash','ZOC');
    console.log(await tx.wait());
    
}

async function deployZypher(){
    const Zypher = await hre.ethers.getContractFactory('RaceZypher');
    const zypher=await Zypher.deploy();
    //const zypher=await Zypher.attach(ZypherAddress);
    const zypherAddress =(String)(await zypher.getAddress());
    console.log(`Zypher deployed to ${(await zypherAddress.toLowerCase())}`);
    await sleep(30_000);
    const tx1=await zypher.setVerifiers(MockShuffleVerifierAddress,MockRevealVerifierAddress);
    console.log(await tx1.wait());
}


async function deployVRF( ){
    let vrf = await hre.ethers.getContractFactory("GameVRF");
    const VRF=  await vrf.deploy(opBNBTestnetVRFAddress);
    const address=(await VRF.getAddress());
    console.log(`VRF deployed to ${address}`)
    
}

async function deployGame(){
    let Game=await hre.ethers.getContractFactory("RaceGame",
        {libraries:{
            IState:LibraryAddress
        }}
    );
    //const game=await Game.attach(GameAddress);
    const game=await Game.deploy(GameCardAddress);
    const gameAddress=(String) (await game.getAddress());
    console.log(`Game deployed to ${gameAddress.toLowerCase()}`);
    await sleep(30_000);
    const Nft= await hre.ethers.getContractFactory('RaceGameCardA',
        {libraries:{
        IState:LibraryAddress
    }}
    );
    const gameCard = await Nft.attach(
        GameCardAddress,
    )
    const tx=await gameCard.setGameContract(gameAddress);
    console.log(await tx.wait());
    // const gasLimit=await game.setVRF.estimateGas(vrfAddress);
    // console.log("gasLimit",gasLimit);
    const tx2=await game.setVRF(VRFAddress);
    console.log(await tx2.wait());
    let VRF= await hre.ethers.getContractFactory("GameVRF");
    let vrf=await VRF.attach(VRFAddress);

    const tx3=await vrf.setGameContractAddress(gameAddress);
    console.log(await tx3.wait());
    
    //Set verifiers
    // const tx4=await game.setVerifiers(MockShuffleVerifierAddress,MockRevealVerifierAddress);
    // console.log(await tx4.wait());
    const tx4=await game.setZypher(ZypherAddress);
    console.log(await tx4.wait());
    
}

async function getCardDetails(){
    const Nft= await hre.ethers.getContractFactory('RaceGameCardA',
        {libraries:{
        IState:LibraryAddress
    }}
    );
    const gameCard=await Nft.attach(GameCardAddress);
    console.log(await gameCard.getAllCards("0x8aD407CEC851382005f2d9B0b664A284bdf0d00D"));
}

async function createSubscription(opBNBVRF: string){
    const testVRFCoordinatorContract = await hre.ethers.getContractFactory(
        'VRFCoordinatorContract',
      )
    const contract = await testVRFCoordinatorContract.attach(
        opBNBVRF,
    )
    try {
        const obj = await contract.createSubscription()
        console.log("obj",obj);
    } catch (e) {
        console.log("error",e);
    //assert.fail('createSubscription failed')
    }
    
}


const testShuffle=async()=>{
    let Game=await hre.ethers.getContractFactory("RaceGame",
        {libraries:{
            IState:LibraryAddress
        }}
    );
    const game=await Game.attach(GameAddress);
    //const tx=await game.initPlayer.estimateGas(1,{value:hre.ethers.parseEther("0.0001")});
    //console.log(await tx);
    // console.log((await game.matches(1)).envDeck.shuffleCount);
    // console.log("check ",JSON.stringify(maskedCards)==  JSON.stringify(maskedCards2));
    const maskedCardsNew: bigint[][]=[];
    var counter=0;
    for(const s of maskedCards3){
        if(counter%4==0){
          maskedCardsNew.push([])
        }
        maskedCardsNew[maskedCardsNew.length-1].push(hre.ethers.toBigInt(s));
        counter++;
    }
    const shuffleCardsNew:bigint [][]=[];
    var counter=0;
    for(const s of shuffledCards3){
        if(counter%4==0){
            shuffleCardsNew.push([])
        }
        shuffleCardsNew[shuffleCardsNew.length-1].push(hre.ethers.toBigInt(s));
        counter++;
    }
    // for (const s of shuffledCards2){
    //     shuffleCardsNew.push([]);
    //     for (const s2 of s){
    //         shuffleCardsNew[shuffleCardsNew.length-1].push(hre.ethers.toBigInt(s2))
    //         if(counter==0){
    //             console.log("bigint ", hre.ethers.toBigInt(s2),s2)
    //         }
    //         counter++;
    //     }
    // }
    // console.log("shuffleCardsNew",shuffleCardsNew);
    console.log("estimate gas",await game.maskEnvDeck.estimateGas(1,maskedCardsNew,shuffleCardsNew,proof3))
    // console.log(await game.getPKC(1));
}
//0x5FbDB2315678afecb367f032d93F642f64180aa3

 
const getVRFDetails=async()=>{
    let VRF= await hre.ethers.getContractFactory("GameVRF");
    let vrf=await VRF.attach(VRFAddress);
    const subId=await vrf.subscriptionId();
    console.log("subId",subId);
    console.log("disableGame",await vrf.disableGame());
    console.log("lastRequestId",await vrf.lastRequestId());
    console.log("requests",await vrf.s_requests(67750703162458150329676888986795115264157074491062109226331865320791089404101n));
    const coordinatorAddress=await vrf.getCoordinatorAddress();
    console.log("coordinatorAddress",coordinatorAddress);
}

const directTopup=async()=>{
    let VRFCoordinatorContract= await hre.ethers.getContractFactory("VRFCoordinatorContract");
    let vrf=await VRFCoordinatorContract.attach(opBNBTestnetVRFAddress);
    //console.log(await vrf.deposit(22,{value:hre.ethers.parseEther("0.0001")}))
    console.log(await vrf.getConfig());
}

const topupVrfSubscription=async()=>{
    let VRF= await hre.ethers.getContractFactory("GameVRF");
    let vrf=await VRF.attach(VRFAddress);
    const tx=await vrf.topUpSubscription({value:hre.ethers.parseEther("0.0005")});
    console.log(await tx.wait());
}
const requestRandomWordsDirectly=async()=>{
    let VRF= await hre.ethers.getContractFactory("GameVRF");
    let vrf=await VRF.attach(VRFAddress);
    // const tx=await vrf.topUpSubscription({value:hre.ethers.parseEther("0.0001")});
    // console.log(await tx.wait());
    //console.log("op BNB coordinator address",await vrf.getCoordinatorAddress());
    //console.log(await vrf.gameContract());
    // console.log(await vrf.setGameContractAddress(GameAddress));
    //console.log(await vrf.setCallbackGasLimit(2500000));
    // console.log(await vrf.topUpSubscription({value:hre.ethers.parseEther("0.0001")}))
    console.log(await vrf.requestRandomWords.estimateGas(
        "0x8aD407CEC851382005f2d9B0b664A284bdf0d00D",
        20,
        2,
        0,
        0 ))
    // try{
    // const tx=await vrf.requestRandomWords(
    //     "0x8aD407CEC851382005f2d9B0b664A284bdf0d00D",
    //     1,
    //     2,
    //     0,
    //     0);
    // await tx.wait();
    // }catch(err){
    //     const {reason}=await errorDecoder.decode(err);
    //     console.log("Error",reason);
    // }
}

const parseError=async(hash: string)=>{

const provider = new hre.ethers.JsonRpcProvider(
    "https://opbnb-testnet-rpc.bnbchain.org"
)
    const data= await  provider.getTransaction(hash);
    console.log("data",data);
}


const getPlayerCards=async(address:string)=>{
    const Nft= await hre.ethers.getContractFactory('RaceGameCardA',
        {libraries:{
        IState:LibraryAddress
    }}
    );
    //const gameCard = await Nft.deploy();
    const gameCard=await Nft.attach(GameCardAddress);
    console.log(await gameCard.getAllCards(address));
    console.log(await gameCard.getPlayerCardProps(address));
}

const checkVRFCoordinatorContract=async()=>{
    const testVRFCoordinatorContract = await hre.ethers.getContractFactory(
        'VRFCoordinatorContract',
      )
      const contract = await testVRFCoordinatorContract.attach(
        opBNBTestnetVRFAddress,
      )

      console.log(await contract.getSubscription(19));
}


// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.

//GameCard 0x4a10D752ab63E489c105060377ded20F4FBF99Ad

//Game 0xe985b561870c2112e6e0F77D21aa11eEa9afccC2

//npx hardhat run --network testnet scripts/deploy.ts
// MockRevealVerifier address 0x24Be0C200d793dA0BDDc1E3Bb8b7f4e9D71D4eB0
// MockShuffleVerifier address 0x96F69e9b398cA1098804B0dc9F4093f2ddBE1926
// createSubscription(opBNBTestnetVRFAddress).catch((error) => {
//     console.error(error);
//     process.exitCode = 1;
//   });
//"0x8f2fab49f17df923faad127d0984df89d01beb9c"
  

    const proof3="0x0f5f8fd568de5b1b64dde00a3520b546b51a1bb62cb1f7886919ebf0a33d0ddc2d2ed7f81d3cd8efa0ea1a7aec9ce73ba5aaa826216af8f5d167ebe86634edcf01655ccd2aec5a08477561a70b0d4b989525f34eaf4d294f161d68f87cdd3c7f1a798cac89bc37f97542dd1a6c23e0c2f2fee40d361e42d94b3ac0582efa0d4c01486ddb4149cc58bc12cc9607ae9cf7c19fc5744ffd098e2e7b491e5dfe58ce057c2d8a5817786e1aeab5d35cf2f93232fbb2c11daec67d903b69ce11a483481946b87986990f3cd2c6e27d7c635be0c15bae4bd6fee46c02943bcedb2cf749014bd3d23f5a13d44aeaae3f89442ee220fa934dbdc76d411717bd25653815030d6a88fdcf548df4e700cf53f54861b307dd581c9964cb6e661a77e5ba9ef10f29516fa787d8812fc12e1df5b1c13c481e19578027ca8b1881c37c9209e168512e6505c9a8ed0f98a97333ff10cff112ce13aea55b644043b065b2b49b9221d322f97bbfd5a8099d9b0bc93ac1b24e514640d99d9512971f37112c0196771a2b00a864780dacda384d64ed9498a18e0aaf982b152429b7f5a35f0603e17dd9a209ba15294a05bea8d707e5ed96335b30402893b9bf55cb885ff077ad415623c31ae9df2d301528748a3166404aeeba0d126c4a84f541e99a4efe7abf8b44d6ab150d9bbdaa9dd711d19636342488b9d1f7cbd6352df8993f7e964f92acfd6efb02991e8ec5e7dd71b365049cf95378708458cdfa1879d2b969948d870c0891871c2dfeb93a1d385f4a7c85f6c717584009ac9959f7d8d320b8621af39ee068592ab3ff3c95e9ec07e95fc8febde9937a0ba438c9e26a131cce9d4decf6883baf289971989ead20b78dbd506559267dd42390b53d01a59798e516560c16242f6e05ba6c9488aeb1b88631d0b32f987ec6ec69084277898e42d10b6b9432a9047108ab2a000228254c9d621d44a6010e50bffc9f034848af746d30d94ea0205b5a0923491a9b90fac61f6e6122feb1dd462738a4b6181ea2e123ea5bf58cc44e6e0634d4d31c8a34a764390e69072de82a06140a8e0a0e603a836b3308dd6069a80cfb5fa0ec0b7e4d9f81ca87d85dc58170ebcecd5ae22a4ed757ff53e8ad93e62a9353c135100272d437c9c672427778d32ce3f5eb2f7afdfb618f01ce03ddf629d466b9c92912520b9abb289f692ffea0a6a858687cd93dd39dae9a907604d629eb28478a5929abd5b7ae4fff67e8dfa004f3059b4b303175d7bd656c61f3ef00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000266b841b6fb6abd2442561d898aefbcf6476ec30a466505e0d0deeca15c6543e15f403a7c8205390b4e4730d66a45c825754811156d4bdcfa092512bc85c2d4815679af7c7bd155f859d2c413a89487a56765939dda03db413cc7d95d86b356414070a0926d2ee19c6ccd040eeadaf3b8333f8e5d57037327e05be73e3463d3501ddb8c301d34e0536afbe7b249ccc58b95090a98f422aefa0e1afe296fe333806f52778cfa33a382458aa4f7c3a1061620481c291560c97259531eb8f340e8708162a84c7a8dffab36ccc66c557f4ce3a04ecaee8e342c21905ebd21834e6c42accd80f857f74122ec51354a6cd15cb9b45ff04357a2e634644bbf8253bba082e85effef1652c75ab4fa5030ff53fd4ba1ef3a8481004795017c3827944dd932a262a1b41b8c65aac798cb8a6d9e229169c6f347c60ecb1846c2be44db930cb0ac8988fac7a58850e03fcf7ce76f339b2ab1ff8d4fc237daed560702df623211b0067c70e642e2952899781fda5fbbd4ec9d56a0260031eb74e993b239c172e1e3319c36a332ced4c272e2b9c3a1e34f0f8071ab09c2170d85f52ffff0541b00223c796f447892b3d1c39bd85d9c61cbb6941b9cb457517acb7b84bfec090cd2fdf8c1405c565745252c701acae28357e72da9f1fa3fad5e10eadbd5ec85ea313013d9c0463ab4be337388e2a8b3caa728443c8c798cdfdc28fdc28b4ea92bb0fd2545d4946c59fd59deaa193d676a5fccaa309f184136f72e7476a71c5b77e2d8fc9e16d2b60eddce7b7ad0bfbb6c3dd426617cdc9862c06be2e04b02980ab1f5c3b6b8cf4e274ff1e8b518b4be92c9498bfee5ae3839fd800fccee28f47d70006866d7edccc2ba4a64a07015f8b7738e4faa3bad4829e99f6f2132f55bf9716b0700fbadc67da6758265a94a3eafc0ccc03fb03f194ff02c493deb001beb7";

    const maskedCards3=[
      "0x218ce9bf8e71ec0e86503b7c5bb6dc41a8555e2f831aa06baa50fa2fecce1cfa",
      "0x27290ca3bad842546c971f8aa3b13b84300fec5ea9d8d2045b055f2f4031abc9",
      "0x2b8cfd91b905cae31d41e7dedf4a927ee3bc429aad7e344d59d2810d82876c32",
      "0x2aaa6c24a758209e90aced1f10277b762a7c1115dbc0e16ac276fc2c671a861f",
      "0x22819234aa9c6645b6982a027eab08fdc736cc3ebd270ffc59e29a3ca85bb4a6",
      "0x257b39c39fd1b30cf4c72b5a549772e53f560c45c3dd8735bbe9d425c9913a6d",
      "0x2b8cfd91b905cae31d41e7dedf4a927ee3bc429aad7e344d59d2810d82876c32",
      "0x2aaa6c24a758209e90aced1f10277b762a7c1115dbc0e16ac276fc2c671a861f",
      "0x2070d8b05ef7e4360c7cd120e7b21aca02df363ff87eefb37b3718b2f73cf987",
      "0x1a0606f39e33eb502189ec1d87de6dc35a24b4b26ecccdcbf1d8fe1540d8e18f",
      "0x2b8cfd91b905cae31d41e7dedf4a927ee3bc429aad7e344d59d2810d82876c32",
      "0x2aaa6c24a758209e90aced1f10277b762a7c1115dbc0e16ac276fc2c671a861f",
      "0x01b489dcbaf3372c8b586ac3f9a1e366333dc158c53440116b3f7380a4aeb87a",
      "0x019dadf830f711e7b65aa6a8b059867188a959d7599becde5ffa5147738b8294",
      "0x2b8cfd91b905cae31d41e7dedf4a927ee3bc429aad7e344d59d2810d82876c32",
      "0x2aaa6c24a758209e90aced1f10277b762a7c1115dbc0e16ac276fc2c671a861f",
      "0x1245544e952faf04fddb969fcd5310a916bbe5fec9e925ef1b7a5d9efce743c6",
      "0x1c3d03003cdc88361800f6e0033e53157cac88bd99f52937e74a476671e54f4f",
      "0x2b8cfd91b905cae31d41e7dedf4a927ee3bc429aad7e344d59d2810d82876c32",
      "0x2aaa6c24a758209e90aced1f10277b762a7c1115dbc0e16ac276fc2c671a861f",
      "0x11c5bd3206781b352bf662197d273b6e921742f8f018400a6b55224dd3529e3e",
      "0x09c2d2981a44eadf340d34c38d04c0134af9c51593ca3836bd463e0ff8c9455f",
      "0x2b8cfd91b905cae31d41e7dedf4a927ee3bc429aad7e344d59d2810d82876c32",
      "0x2aaa6c24a758209e90aced1f10277b762a7c1115dbc0e16ac276fc2c671a861f",
      "0x25d0b32f3611bc80cd293fda04e398d7fe97cf08fffaa2dc1857ce7e54caa900",
      "0x08ca5c823686ae0e899a19f6a02ba9433b9aff14d38b6b2d2a4e2e77b35bb09a",
      "0x2b8cfd91b905cae31d41e7dedf4a927ee3bc429aad7e344d59d2810d82876c32",
      "0x2aaa6c24a758209e90aced1f10277b762a7c1115dbc0e16ac276fc2c671a861f",
      "0x03fea0d63e1b2638e629e4a62851358de6756ec2654929435a83ac79e27d9d76",
      "0x0f4057aa149c2789580ca13e13f58f5bc560401de0cf84fe40245797690dcb64",
      "0x2b8cfd91b905cae31d41e7dedf4a927ee3bc429aad7e344d59d2810d82876c32",
      "0x2aaa6c24a758209e90aced1f10277b762a7c1115dbc0e16ac276fc2c671a861f",
      "0x08c194c6edfa0e5817e2512ed9fa1f9a31176f33054c43a0f09c82921b277d11",
      "0x02cf1852c4d3c70254925809cc3b3e471352cae7cd7a02972a4fbc25e49ff8d2",
      "0x2b8cfd91b905cae31d41e7dedf4a927ee3bc429aad7e344d59d2810d82876c32",
      "0x2aaa6c24a758209e90aced1f10277b762a7c1115dbc0e16ac276fc2c671a861f",
      "0x2b76968337a76cd315ff36759c2afed4156079ce8d03bf8e1a99d56af60dfee4",
      "0x12804496f72a370a83a3c0e200c0406768a4865c6c966b1048831800b4bdfa04",
      "0x2b8cfd91b905cae31d41e7dedf4a927ee3bc429aad7e344d59d2810d82876c32",
      "0x2aaa6c24a758209e90aced1f10277b762a7c1115dbc0e16ac276fc2c671a861f",
      "0x163a252b384d0bc1c62ace71d7c98d833d0aa5596f152a90615a26a7a39ae3ec",
      "0x164ca5f144bbc192e71168dbc0619ec1f570634fdfb72b01c88039f75596f10d",
      "0x2b8cfd91b905cae31d41e7dedf4a927ee3bc429aad7e344d59d2810d82876c32",
      "0x2aaa6c24a758209e90aced1f10277b762a7c1115dbc0e16ac276fc2c671a861f",
      "0x07b06e605534b3e89ed492a5017a62eb5753c2d64357feb4118a8e057cb8c2ae",
      "0x1161d9cd4e7180afbf6c65add16bf1bc8cc3e3774d93341ddd265a2739e03fca",
      "0x2b8cfd91b905cae31d41e7dedf4a927ee3bc429aad7e344d59d2810d82876c32",
      "0x2aaa6c24a758209e90aced1f10277b762a7c1115dbc0e16ac276fc2c671a861f",
      "0x2be54ccd8a27e7196eafa05d387ad2748fc828aa7f8230668eb598b41d692a6e",
      "0x1733013b4152066f4a38b6c75484b7669c71418ca59f7ac32e9c1c393f3b3738",
      "0x2b8cfd91b905cae31d41e7dedf4a927ee3bc429aad7e344d59d2810d82876c32",
      "0x2aaa6c24a758209e90aced1f10277b762a7c1115dbc0e16ac276fc2c671a861f",
      "0x2b4ed7d39ae55cd435fda64709e2d6ce9a5d7d8519a2e6e6a8f71e80d83942b7",
      "0x2d9a5a442767f61165dad8677c89abaa167b35a67b0baa1f5b6a871aa97165ba",
      "0x2b8cfd91b905cae31d41e7dedf4a927ee3bc429aad7e344d59d2810d82876c32",
      "0x2aaa6c24a758209e90aced1f10277b762a7c1115dbc0e16ac276fc2c671a861f",
      "0x04aacf4835d4c2d46260ef14fa8539566eaf58e5d00c71b1b80fa5e3bc522976",
      "0x2c5746f4ba6c4dc80147c39e518dfeca2faab87182d194a541fb9942b285685f",
      "0x2b8cfd91b905cae31d41e7dedf4a927ee3bc429aad7e344d59d2810d82876c32",
      "0x2aaa6c24a758209e90aced1f10277b762a7c1115dbc0e16ac276fc2c671a861f",
      "0x12cfe6586c85accf57b0a4cc08fa5c9c307f71bdd6fac33b181292fde4f05395",
      "0x29d54126f5269094fb9f59e3e2ce5ea8c100af040a372a62f35d75c1f2105093",
      "0x2b8cfd91b905cae31d41e7dedf4a927ee3bc429aad7e344d59d2810d82876c32",
      "0x2aaa6c24a758209e90aced1f10277b762a7c1115dbc0e16ac276fc2c671a861f",
      "0x0c9ca759a8334bb3691b7d106e0a8dfef33d59e62d5dea7581ecef987b367ed5",
      "0x20c9b1fb4a0999d297372e401d0376909592433fec0c507250bd4dc68d5a2322",
      "0x2b8cfd91b905cae31d41e7dedf4a927ee3bc429aad7e344d59d2810d82876c32",
      "0x2aaa6c24a758209e90aced1f10277b762a7c1115dbc0e16ac276fc2c671a861f",
      "0x2aa4a6dd47dcef8ba03c5779761d13e89e261fa138f2251becf9e48cfd482a27",
      "0x24e6bfa0b8e7534f65c02d03e1c2cb0c07fcce8e6b938c0b559b076636f283a5",
      "0x2b8cfd91b905cae31d41e7dedf4a927ee3bc429aad7e344d59d2810d82876c32",
      "0x2aaa6c24a758209e90aced1f10277b762a7c1115dbc0e16ac276fc2c671a861f",
      "0x0e49c4014b9fb094c591c9f60b2c1353006ce06a044929307ed842c09f443d92",
      "0x102fd4ae1270abb425d492ea44bcae028ca21afba81fd58a1533e382f99d58a8",
      "0x2b8cfd91b905cae31d41e7dedf4a927ee3bc429aad7e344d59d2810d82876c32",
      "0x2aaa6c24a758209e90aced1f10277b762a7c1115dbc0e16ac276fc2c671a861f",
      "0x0e00e37a909c62c8202a8e730053c66078f0a95fa0c3561c5b1d3f16a0d6ddd6",
      "0x1fdb4022b40dd1c88c830c1268f1bffe2f09b07e09b8a6b8bac6f30d4bec45e0",
      "0x2b8cfd91b905cae31d41e7dedf4a927ee3bc429aad7e344d59d2810d82876c32",
      "0x2aaa6c24a758209e90aced1f10277b762a7c1115dbc0e16ac276fc2c671a861f"
  ]

  const shuffledCards3=[
    "0x126c3ad69558a2c31d92d8b5afb2adf795118af4597d1abc147ec20bba330100",
    "0x22d516db7225c3033440f811ba71bf794bb53ddd912d8ecf75aa14be473267b1",
    "0x0f102b6334214077988b76a29e7edc4894b34416be5372ed60a367763011f809",
    "0x2d7c2914decb73a58d553cb81aef7f2fd3ec1a69f7412d521b5523d7a7cc968c",
    "0x2ef9ac59a1127227f0fe829de5622ac8cb71d12cdd469bdd4287988143f33df0",
    "0x22e1e0c198fa10bd6f10136d0914cceeb838ddfde6bad9255304f4ce43c66869",
    "0x2d821097fd85e6b8a7ec19f41cab39f82a75f810b3c02d03e4487b4f2f8a429a",
    "0x0fab9228c24d1972faa7437e49df1c7105b50a6f0483d702c978aa3f79c820b1",
    "0x28f1f263f1e2db2dcd4ff8cef842eee62a690fc9b1cd67fb7246192513dfe8a6",
    "0x03858f4aa0fc5a4f80633a6bfc810a23489195326add29a929ae4cb08070ef6e",
    "0x2c184e21d1b82cd6e78d23064e1895f7e2a60e201792df457c9a79860c6c93af",
    "0x17c27abbef78a25716506315401d3ef29e93266553059141347d085c0d165f6a",
    "0x01319ccc48eacc5531ad16fda68ea82255a694b775deccde975b32b3a1d1bdf8",
    "0x16dbf6268b70bb9e7ddbb4d43658268e5ce850c5dfa4463c376d83337efd8654",
    "0x2399de2f1f9eccb776bd5b72d50e1c3f04ad340e4296bb77c42accd10e507feb",
    "0x180c5ed3427a79daade4fe456856384b580c12e747a5b50d1c988b96e625828e",
    "0x1b10ddfb6f1dbc949fbff5c8c5c22d1e342dbb7b98eefa2210caad72e9a307ab",
    "0x2f49afd38040b460cf0452ae6fac2bc369faca1eb457a18b71b76a6d7c313876",
    "0x0de0adcea4c92844123ce9ee1c6eaf44f25e7dc5cd8ef140f4952748e194796e",
    "0x1f21bf0a9f542afa2a220e936c19ed305f72383ec5e0d780d76df8813d55b76b",
    "0x2f255469943b228adacb80268b8e4f6739a4520f5616ed0378ee221f0fb2294e",
    "0x1d3d7ccbef47f96e1db711437a9b01da933c6f8f2862c1427bf2b081d18677c4",
    "0x1f20c9f09f31339ce446f8dd1e6c28560c07621e7819b0dc41bf4909d2222d6e",
    "0x1761d2b12fb0dd10c39cbb3ba60f82f97ed6e55fbe1f6d4a9f6b445046803b9f",
    "0x0ecaaf38cdf435b84d7051d6bff149b5b144031594bef77b8c46333679abf77d",
    "0x2bd0f946e8140aa08b646baa780bb49caadfc3a99fd07764c3e720f1237a1ef6",
    "0x2113285f1d78bd60c7c0d6da5e6feec78a6a47df19622e9dafa6a65c47f57d7c",
    "0x1b1d1246581c62dde69aa116f21283ee4335a93a026f2bd4b35c0b724103288e",
    "0x2fab318954e61c952dc893d4e6f73a5b8fc37c139da80b77b096424beecef946",
    "0x03e3bdb7a343597f2aca5df176e8fe68d7afd6842992e2a236b7ac71eb36f18f",
    "0x2e8fa5d1c98b54d42988d29029b4d548ef4fe97817e7b9c857dabb47ff2476ec",
    "0x134216210d456be98a264375f6665251072a00cbc8d6c86c4d678660e33a7153",
    "0x0ff7bd8a3dd3ab90b053694e077e1e53561bb9c76bf10a5015e24672218c1d12",
    "0x2091b7dec216a28c2b1bf75b1f26e313b6c1e58918db95c0d446a08adfc208e6",
    "0x051c2eb928b2c5e19550d6a1e978cbf622ac5546d7ee6f3b4dff77d40a83a493",
    "0x17452db310ce1f8698c0dec5e17614662a2e330460d6418072afcce914b658dc",
    "0x089b14948d52a708c9e273ff73269f738faafb7cf57e83c17ab2caa11ac86338",
    "0x249acb6aad649a21330267ac90dd4e40066d295c591576dfc330ccb4be21c0d6",
    "0x26d2e28239ddfce91a626a9268d6dfb15b887b2456a4085c3a5561d8bc7c29c8",
    "0x1d64375a6d0ac5a8ce61662565bf7d9111af67f536f7330454ce5a5e32d30a43",
    "0x1741d547f94067aefdf594a50e4b40448cc29e109f7aa7b79658a100ae8b21f8",
    "0x2ef21aa43ded37f8882565af7f6b44679a676fb28604b8312a50bf689d851555",
    "0x0f23062b08162b10d5f4ab086a3c46987e3193e2d735e9f22a5956837eb6b4f8",
    "0x1b2ba9ec4166cf58b12802068f8be4982beb08f73493672dbea78e9a17d71bf0",
    "0x07581a1f7477079e8b2b136e702edc807cacfd70df993037a07b0b5600896ce6",
    "0x0825325fd0fcb40f8bca87e661ea5e8a80d4b8af86729554d67494991964c3e1",
    "0x1bdbd526835872fa790b85c225706ad9be7e43f9abd9fb4dc25a7cbb9c3f09bf",
    "0x2729038b52f76c123183c4271a828506aca7d17b9ebd42f277e42422f5e1d1e2",
    "0x08732120842bf6aa647c83aefccad4baacac1c5dbbec627288b505b77f1af85b",
    "0x05120495d0bd5d4c792f16ae9cba2f4f379f2adca53ab8bc3f752e21d6e60bae",
    "0x068d94bfb1eed6cb1b663be3aef7f684dfecc2e2274353341c1d575d9d72f7f7",
    "0x1846df495d55df9524d8f501e454c59100c5adcf16edc3f7e27aef05e7ff0435",
    "0x141ac6cbcac1f1013b9af41eed872c126e37a5151ae5fc0405d8a65ca9fc0f23",
    "0x192cb2debe57e854918c21e293fb00a0735318a4d12fbd2fa5568f4cd2b28efe",
    "0x28fe3f221a007cbabd8c16fe45415b5cf66a7eb9ef9dd3611b1e069bc448fabf",
    "0x091e674be914457a2c471eab1bb337d4cf7501ac86fb4f23f5080427cd9c0f9f",
    "0x2c01fae89e6c267a6c02d2b457a8e566df2ab05b254d1812f1cb4d72014d163d",
    "0x04d2f0fe3727e3fce92e15d058fb25808945db9dc9999d866e7c0dd807b6768d",
    "0x22437ac4ea726c7d624b92ba51a83d436e8aa4f9a48ca6b6f3662ab9927a52e2",
    "0x16aa7a0c64ccf81fdf1d7ed90f56da064bdc1a835cdf9e5c99773cb10da5e8c3",
    "0x256801a3589bdbeddb225d89f5e2685aa26b8b87c186b63fe018d2accd897523",
    "0x27a67c007b02dc2fc321b49d31d83f0afe2a488702b67ff432cae17a4510d549",
    "0x029b297f9c6fa5fbf3d86ffca4771d15e0bf8f76b73fa77adf86d5ddda4c4b03",
    "0x3055c36d310f4344eeb1c6decaa3aa988b797d8cd065146356beae252dfde551",
    "0x0f027265cca8158c9fdaeae611f89dcebaba91497ee538afa5bf16106eae4821",
    "0x0b5ed6489ed838aca1bea3b2e3b825a943a703397ed5455580817ed1598e8069",
    "0x0795ded93b377ee78ba12df3a236b7dc117069d1eca26229c5f3cbdc18eaf9c8",
    "0x20de7bdb6311ea14a466a8951c8409a70ccd388974460a3d3d4ed45dbc2d4bc3",
    "0x1701b94f20a0f1118a6cb35c4cff1bd788fed44debc0b47dc4831d12c76ccbb2",
    "0x0a06377f1c6d313fb7fd30c4fb3e7dba1376ae48c9e5722f63c77c3ad5db607c",
    "0x2c18343dbf7a6977e9e8f34d64ca58f7328bf83ab986d707ade3c37d510212e4",
    "0x04cf04c7cba3f5f198915486920b8a01d2a34464c27323f96963505ced280f3b",
    "0x1f1ec3722f2a7f8071a2fc8acb1efeff52243ceea8ffafa0fe5561015b0695f4",
    "0x258dd61f07925f28a9fd64ff739e1d81fb2821fa88dffae325e702e584ac1ecf",
    "0x15c9683976c5c0c9d9eb6393e68421c71e0bfa8075d8734a6770941a5a686063",
    "0x2b6c4d3e1782841ec63540063419adfd25bdebe733a61d225ef640ffd8ebce70",
    "0x27e633e9c3d6ac722262565ec55c85edb72fd859b98bc66757f89cc180ff367e",
    "0x12886e4e7cbc846be00ed0295e44da140162ff1890b9745565cb2d0d1fa4955c",
    "0x283f89a4117fb3f0167680258dccdc6fb2bf386113bc79b1f86a4e81459688e0",
    "0x2f90357316c7f9f6fb7b3872e6e82d82fdc0e331d02c83bb208c30ba84280dc9",
];  

deployGame().catch((error) => {
          console.error(error);
          process.exitCode = 1;
        });

