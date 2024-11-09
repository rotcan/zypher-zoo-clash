import { HardhatEthersSigner } from "@nomicfoundation/hardhat-ethers/signers";
import { RaceGameCardA } from "../typechain-types/contracts/RaceGameCardA.sol";
import { expect } from "chai";
import * as hre from "hardhat";
import { IState, RaceGame, RevealVerifier, ShuffleService, GameVRF, VRFCoordinatorV2Mock, RaceZypher } from "../typechain-types";
import { BigNumberish, ContractTransactionResponse } from "ethers";
import {anyValue} from "@nomicfoundation/hardhat-chai-matchers/withArgs";
import * as SE from '@zypher-game/secret-engine';

const deckSize=20;
type Contract<T> = T & { deploymentTransaction(): ContractTransactionResponse; }; 
type RevealEnvCardType={game: Contract<RaceGame>,
    wallet: HardhatEthersSigner, matchIndex: number,
    sk:string, cardIndex: bigint,deck: [bigint,bigint,bigint,bigint][]
};

type RevealOpponentCardType={game: Contract<RaceGame>,
    wallet: HardhatEthersSigner, matchIndex: number,revealCount:number,revealIndex:number,playerIndex:number,
    sk:string,  deck: [bigint,bigint,bigint,bigint][]
};
describe('init',()=>{

    let MockRevealVerifier: Contract<RevealVerifier>;
    let MockShuffleVerifier:Contract<ShuffleService>;
    let GameCard: Contract<RaceGameCardA>; 
    let VrfCoordinatorV2Mock: Contract<VRFCoordinatorV2Mock>;
    let VRF: Contract<GameVRF>;
    let GameMock: Contract<RaceGame>;
    let MockIState: Contract<IState>;
    let RaceZypher: Contract<RaceZypher>;
    async function loadStateLibrary(){
        const IState= await hre.ethers.getContractFactory("IState");
        MockIState=await IState.deploy();
    }
    async function createMockVerifiers(){
        const deck_num = deckSize;
        const RevealVerifier=await hre.ethers.getContractFactory('RevealVerifier');
        MockRevealVerifier=await RevealVerifier.deploy();

        const VerifierKeyExtra1_20=await hre.ethers.getContractFactory("VerifierKeyExtra1_20");
        const verifierKeyExtra1_20=await VerifierKeyExtra1_20.deploy();

        const VerifierKeyExtra2_20=await hre.ethers.getContractFactory("VerifierKeyExtra2_20");
        const verifierKeyExtra2_20=await VerifierKeyExtra2_20.deploy();
        
        const ShuffleVerifier=await hre.ethers.getContractFactory('ShuffleService');
        MockShuffleVerifier=await ShuffleVerifier.deploy(await verifierKeyExtra1_20.getAddress(),
            await verifierKeyExtra2_20.getAddress(),deck_num);

        //return {MockRevealVerifier,MockShuffleVerifier};
    }

    async function loadGameCard(){
        const Nft= await hre.ethers.getContractFactory('RaceGameCardA',
            {libraries:{
            IState:(await MockIState.getAddress())
        }}
        );
        GameCard = await Nft.deploy();
        GameCard.initialize('AnimalRace','ZAR');
       
        //return {nft}
    }

    async function loadZypher(){
        let raceZypherMock = await hre.ethers.getContractFactory("RaceZypher");
        RaceZypher=await raceZypherMock.deploy();
    }

    async function loadVRFMock(){
        let vrfCoordinatorV2Mock = await hre.ethers.getContractFactory("VRFCoordinatorV2Mock");
        VrfCoordinatorV2Mock=await vrfCoordinatorV2Mock.deploy(0,0);
        // await VrfCoordinatorV2Mock.createSubscription();
        // await VrfCoordinatorV2Mock.fundSubscription(1, hre.ethers.parseEther("7"))
        //return {VrfCoordinatorV2Mock};
    }
    
    async function loadVRF(){
        const address=(await VrfCoordinatorV2Mock.getAddress());
        let vrf = await hre.ethers.getContractFactory("GameVRF");
        VRF=await vrf.deploy(address);
        await VRF.topUpSubscription({value:hre.ethers.parseEther("0.01")});
        // await VRF.setSubscription(1);
        // await VrfCoordinatorV2Mock.addConsumer(1,(await VRF.getAddress()));
        
    }

    async function loadGame() {
        let Game=await hre.ethers.getContractFactory("RaceGame",
            {libraries:{
                IState:(await MockIState.getAddress())
            }}
        );
        GameMock=await Game.deploy((await GameCard.getAddress()));

        await GameCard.setGameContract((await GameMock.getAddress()));
        await GameMock.setVRF((await VRF.getAddress()));
        await VRF.setGameContractAddress((await GameMock.getAddress()));
        await GameMock.setZypher((await RaceZypher.getAddress()));
    }

    async function shuffleOpponentDeck({ matchIndex,otherPlayer, game, wallet, gameKey, deck }:
        {matchIndex:number,otherPlayer:string, 
            game : Contract<RaceGame>, wallet: HardhatEthersSigner,  gameKey: string, deck: [BigNumberish,
                BigNumberish,BigNumberish,BigNumberish][] }) {

        const hexifiedDeck = deck.map((cardBNs: [BigNumberish,
            BigNumberish,BigNumberish,BigNumberish]) => cardBNs.map(bn => hre.ethers.toBeHex(bn)))
      
        const {
          cards: shuffledCards,
          proof,
        } = SE.shuffle_cards(gameKey, hexifiedDeck)
      
        const deckArray:[BigNumberish,BigNumberish,BigNumberish,BigNumberish][]=[];
        for(const m of deck){
            const m2=Array.from(m);
            deckArray.push([m2[0],m2[1],m2[2],m2[3]]);
        }
        return game.connect(wallet).shuffleOtherDeck(matchIndex, otherPlayer,deckArray,shuffledCards, proof)
    }


    async function submitSelfDeck({ matchIndex, game, wallet, gameKey, deckSize }:
        {matchIndex: number,
             game : Contract<RaceGame>, wallet: HardhatEthersSigner, gameKey: string, deckSize: number }
    ) {
        const maskedCards = SE.init_masked_cards(gameKey, deckSize)
                              .map(({ card }:{card:any}) => card)
        const {
          cards: shuffledCards,
          proof,
        } = SE.shuffle_cards(gameKey, maskedCards)
        // console.log("maskedCards",(await wallet.getAddress()),maskedCards);
        // console.log("shuffle_cards",(await wallet.getAddress()),shuffledCards);
        return game.connect(wallet).submitDeck(matchIndex, maskedCards, shuffledCards, proof)
    }

    async function submitEnvInitDeck({ matchIndex, game, wallet, gameKey, deckSize }:
        {matchIndex: number,
             game : Contract<RaceGame>, wallet: HardhatEthersSigner,   gameKey: string, deckSize: number }){
        const maskedCards = SE.init_masked_cards(gameKey, deckSize)
                              .map(({ card }:{card:any}) => card)
        
        const {
          cards: shuffledCards,
          proof,
        } = SE.shuffle_cards(gameKey, maskedCards)
         
        //shuffleCardsNew.map(m=>console.log("m",m));
        expect(SE.verify_shuffled_cards(maskedCards,shuffledCards,proof)).eq(true)
        return game.connect(wallet).maskEnvDeck(matchIndex,  maskedCards, shuffledCards, proof)
    }


    async function submitEnvShuffleDeck({ matchIndex, game, wallet, gameKey, deck }:
        {matchIndex: number,
             game : Contract<RaceGame>, wallet: HardhatEthersSigner,   gameKey: string, deck: [BigNumberish,
                BigNumberish,BigNumberish,BigNumberish][]  }){
       
        const hexifiedDeck = deck.map((cardBNs: [BigNumberish,
            BigNumberish,BigNumberish,BigNumberish]) => cardBNs.map(bn => hre.ethers.toBeHex(bn)))
        
        const {
            cards: shuffledCards,
            proof,
        } = SE.shuffle_cards(gameKey, hexifiedDeck)
        
        const deckArray:[BigNumberish,BigNumberish,BigNumberish,BigNumberish][]=[];
        for(const m of deck){
            const m2=Array.from(m);
            deckArray.push([m2[0],m2[1],m2[2],m2[3]]);
        }
        return game.connect(wallet).shuffleEnvDeck(matchIndex, deckArray ,shuffledCards, proof)
    }


    async function reveaEnvCard({ matchIndex,game, wallet, sk,  deck, cardIndex }
        :{matchIndex: number,game: Contract<RaceGame>, cardIndex:bigint,
            wallet: HardhatEthersSigner, sk: string,  deck: bigint[][],}
    ) {
        const target = deck[+cardIndex.toString()].map(bn => hre.ethers.toBeHex(bn))
        
        const {
          card: revealToken,
          snark_proof: proof,
        } = SE.reveal_card_with_snark(sk, target)
      
        await game.connect(wallet).showNextEnvCard(matchIndex, revealToken, proof)
      }

    
    async function showOpponentHand({ game, wallet, sk, revealCount,revealIndex, matchIndex, deck,playerIndex }:
        { game : Contract<RaceGame>, wallet: HardhatEthersSigner, sk: string,revealCount: number,revealIndex:number,  
            deck: bigint[][],matchIndex:number,playerIndex: number }) {
        const revealTokens=[];
        const proofs=[];
        for (var i=0;i<revealCount;i++) {
          const target = deck[revealIndex+i].map(bn => hre.ethers.toBeHex(bn))
          const {
            card: revealToken,
            snark_proof: proof,
          } = SE.reveal_card_with_snark(sk, target)
          revealTokens.push(revealToken);
          proofs.push(proof);
        }
        await game.connect(wallet).showOpponentCards(matchIndex,playerIndex,revealCount, 
            revealTokens, proofs)
    }


    async function printHandCards({ name, deck, sk, hands, rTokens, nfts }:
        {name: String, deck: bigint[][], sk: string,hands: number[],rTokens: bigint[][][],nfts:bigint[]}): Promise<number[]> {
        const indexes = hands.map(idx => SE.unmask_card(
          sk,
          deck[idx].map(bn => hre.ethers.toBeHex(bn)),
          rTokens[idx].map(bns => bns.map(bn => hre.ethers.toBeHex(bn)))
        ))
        
        const nftIds = indexes.map(idx => +nfts[idx].toString(10))
      
        return nftIds;
    }
    

    async function playCardTxn({ game, wallet, sk, hand, deck, handIndex,matchIndex }
        :{ game: Contract<RaceGame>, wallet: HardhatEthersSigner, sk: string, matchIndex:number,
            hand: number[], deck: bigint[][], handIndex: number }
    ) {
        const cardIndex = hand[handIndex]
        const target = deck[cardIndex].map(bn => hre.ethers.toBeHex(bn))
      
        const {
          card: revealToken,
          snark_proof: proof,
        } = SE.reveal_card_with_snark(sk, target)
      
        await game.connect(wallet).playCardOnDeck(matchIndex, handIndex, revealToken, proof)
      }

    async function playCard({matchIndex,playerIndex,keys,revealCount,players,
        handIndex,    
    }:{matchIndex:number,playerIndex: number,handIndex:number,
        keys: {
            sk: string;
            pk: string;
            pkxy: string[];
        }[],revealCount:number,
        players: HardhatEthersSigner[],
    }){
        let playerData=(await GameMock.getPlayerDataByIndex(matchIndex,playerIndex));
        expect(playerData.playerRevealCount).not.eq(playerData.nextRoundPlayerRevealCount);
        for(var i=0;i<players.length;i++){
            if(i!==playerIndex){
                const player=players[i];
                await showOpponentHand({game: GameMock, sk: keys[i].sk,
                    revealCount, revealIndex:+playerData.playerRevealCount.toString(), matchIndex:matchIndex, 
                    playerIndex, deck: playerData.playerDeck,
                    wallet: player,
                })
            }
        }
        
        playerData=(await GameMock.getPlayerDataByIndex(matchIndex,playerIndex));
        expect(playerData.playerRevealCount).eq(playerData.nextRoundPlayerRevealCount);
        await playCardTxn({game: GameMock, wallet:players[playerIndex],
             sk: keys[playerIndex].sk, 
             hand: playerData.playerHand.map(m=>+m.toString(10)),
            handIndex, deck: playerData.playerDeck, matchIndex
        })
    }

    async function playTurn({keys,matchIndex,players,
        revealEnv
    }:{
        matchIndex:number,revealEnv: number,
        keys: {
            sk: string;
            pk: string;
            pkxy: string[];
        }[],
        players: HardhatEthersSigner[],
    }){
        let matchData=(await GameMock.matches(matchIndex));
        const playerTurn=+matchData.playerTurn.toString();
        const rounds=+matchData.rounds.toString();
        for(var i=0;i<players.length;i++){
            const currentPlayerIndex=(i+playerTurn)%players.length;
            matchData=(await GameMock.matches(matchIndex));
            const playerData=await GameMock.getPlayerDataByIndex(matchIndex,currentPlayerIndex);
            const initRevealCount=playerData.playerRevealCount;
            if(revealEnv === currentPlayerIndex){
                //Reveal env
                await GameMock.connect( players[currentPlayerIndex]).playerAction(matchIndex,1);
                const envDeck=(await GameMock.matches(matchIndex)).envDeck;
                const envRevealIndex= envDeck.envRevealIndex;
                const arr:RevealEnvCardType[]=[];
                for(var j=0;j<players.length;j++){
                    arr.push({game:GameMock, wallet: players[j],  matchIndex, sk:keys[j].sk,
                            cardIndex:envRevealIndex,
                            deck:envDeck.cards })
                }
                await Promise.all(arr.map(reveaEnvCard));

                expect((await GameMock.getPlayerDataByIndex(matchIndex,currentPlayerIndex)).playerRevealCount).eq(initRevealCount);
                expect((await GameMock.matches(matchIndex)).envDeck.envRevealIndex).eq(envRevealIndex+1n);
            }else{
                if(matchData.turnStart==0n)
                    await GameMock.connect( players[currentPlayerIndex]).playerAction(matchIndex,0);
                //Reveal card
                // const arr:RevealOpponentCardType[]=[];
                // for(var j=0;j<players.length;j++){
                //     if(j!=currentPlayerIndex){
                //         arr.push({game: GameMock, sk: keys[j].sk,
                //             revealCount:1, revealIndex:+playerData.playerRevealCount.toString(), matchIndex:matchIndex, 
                //             playerIndex:currentPlayerIndex, deck: playerData.playerDeck,
                //             wallet: players[j],
                //         })
                //     }
                   
                // }
                // await Promise.all(arr.map(showOpponentHand));
                //Play Card
                await playCard({matchIndex,playerIndex:currentPlayerIndex,handIndex:i+1,keys,players,revealCount:1});
                expect((await GameMock.getPlayerDataByIndex(matchIndex,currentPlayerIndex)).playerRevealCount).eq(initRevealCount+1n);
            }
            
        }
        matchData=(await GameMock.matches(matchIndex));
        expect(matchData.rounds).eq(rounds+1);
        for(var i=0;i<players.length;i++){
            console.log("Round ("+matchData.rounds+") score p("+(i+1)+") : ",
            (await GameMock.getPlayerDataByIndex(matchIndex,i)).position);
        }
    }
 
    beforeEach(async()=>{
        // const [owner] = await hre.ethers.getSigners();
        // const {MockIState} = await loadStateLibrary();
        await loadStateLibrary();
        await createMockVerifiers();
        // const  {nft}=await loadGameCard((await MockIState.getAddress()));
        await loadGameCard();
        await loadZypher();
        await loadVRFMock();
        await loadVRF();
        await loadGame();
    })

    it('Test State',async()=>{
        const val=await MockIState.createCard(Number.parseInt(""+Math.random()*1000000),Number.parseInt(""+Math.random()*1000000),2);
        const card_prop = val[0];
        console.log("card_prop",card_prop);
        
        console.log("steps",card_prop & 7n);
        console.log("favoredGeographies",card_prop >> 3n & 127n);
        console.log("rarity",card_prop,card_prop >> 11n);
        const r1=Number.parseInt(""+Math.random()*1000);
        const r2=0;
        const t2=await MockIState.getRarity(r1,r2);
        const t3=await MockIState.getEnvCard(Number.parseInt(""+Math.random()*1000000),Number.parseInt(""+Math.random()*1000000));
        const score = await MockIState.calculateAnimalScore(val,t3);
        console.log("envCard",t3)
        console.log("rarity test ",t2,r1,r2);
        console.log("score",score);
        // const allcards=await MockIState.getAllAnimalCards();
        // console.log("allcards",allcards);
    })
 

    xit('Game test',async()=>{
        expect(1).to.be.equal(1,"What the hell")
        //console.log("MockRevealVerifier ",(await MockRevealVerifier.getAddress()))
        //console.log("GameMock ", (await GameMock.getAddress()));
        const [owner,p1,p2]=await hre.ethers.getSigners();
        const players=[p1,p2];
        //Check card count
        expect(await GameCard.getAllCards(p1.address)).have.lengthOf(0);
        expect(await GameCard.getAllCards(p2.address)).have.lengthOf(0);

        // console.log("Game address",(await GameMock.getAddress()));
        // console.log("GameCard address",(await GameCard.getAddress()));
        // console.log("VRF address",(await VRF.getAddress()));
        const nftCount=await GameMock.INIT_MINT_COUNT();
        //Init Player
        await expect(GameMock.connect(p1).initPlayer(1,{value:hre.ethers.parseEther("0.0001")})).to.emit(GameMock,"RequestSent").withArgs(1,nftCount);
        await expect(
            VrfCoordinatorV2Mock.fulfillRandomWords(1, (await VRF.getAddress()))
        ).to.emit(VrfCoordinatorV2Mock, "RandomWordsFulfilled").withArgs(1,anyValue,anyValue,true);
        
        expect(await GameCard.getAllCards(p1.address)).have.lengthOf(nftCount);

        await expect(GameMock.connect(p2).initPlayer(1,{value:hre.ethers.parseEther("0.0001")})).to.emit(GameMock,"RequestSent").withArgs(2,nftCount);
        await expect(
            VrfCoordinatorV2Mock.fulfillRandomWords(2, (await VRF.getAddress()))
        ).to.emit(VrfCoordinatorV2Mock, "RandomWordsFulfilled").withArgs(2,anyValue,anyValue,true);
        expect(await GameCard.getAllCards(p2.address)).have.lengthOf(nftCount);

        const winningScore=12;
        const p1Cards=await GameCard.getPlayerCardProps(p1.address);
        const promises= p1Cards[1].map(m=>MockIState.getAnimalRarity(m[0]));
        const p1rarities=(await Promise.all(promises)).join(",")
        console.log("p1rarities",p1rarities);
        //hre.ethers.AbiCoder.defaultAbiCoder().decode(["uint8", "uint8", "uint8", "address", "uint256"],);
        //Set ZG verifiers
        const shuffler= MockShuffleVerifier;
        const revealer = MockRevealVerifier;
        await RaceZypher.setVerifiers(
            await shuffler.getAddress(),
            await revealer.getAddress()
        )
         
        const keys=[{ 
            sk: "0x020b31a672b203b71241031c8ea5e5a4ef133c57bcde822ac514e8a1c7f89124", 
            pk: "0xada2d401ec3113060a049b5472550965f59423eaaeec3133dd33628e5df50491", 
            pkxy: ["0x27f9bc87a7fe674c14532699864907156753a8271a6e97b8f8b99a474ad2afdd",
                 "0x1104f55d8e6233dd3331ecaeea2394f565095572549b040a061331ec01d4a2ad"] },
                 { sk: "0x02d75fed474808cbacf1ff1e2455a30779839cfb32cd79e2020aa603094b80b7", 
                    pk: "0x52bd82819071b9b913aacfccc6657e5226d1aebd5e5ec4fbdea0b6f5bb2bdf12", 
                    pkxy: ["0x0fc2c87764783cdc883744c16712654ce3d0fccbea70c9ce379a8bc7f412f006", 
                        "0x12df2bbbf5b6a0defbc45e5ebdaed126527e65c6cccfaa13b9b971908182bd52"] }

        ];

        // console.log("p2.address",p2.address,"p1.address",p1.address,"owner.address",_owner.address);
        //Create match
        await expect(GameMock.connect(p1).createNewMatch({x: keys[0].pkxy[0], y: keys[0].pkxy[1]},2,winningScore,1,{value:hre.ethers.parseEther("0.0001")}))
            .to.emit(GameMock,"RequestSent").withArgs(3,nftCount);
        await expect(
            VrfCoordinatorV2Mock.fulfillRandomWords(3, (await VRF.getAddress()))
        ).to.emit(VrfCoordinatorV2Mock, "RandomWordsFulfilled").withArgs(3,anyValue,anyValue,true)
        .to.emit(GameMock, "EnvDeckCreated").withArgs(nftCount);
        const matchIndex=1;
        
        const p1CardIndices=Array.from(await GameCard.getAllCards(p1.address)).slice(0,deckSize);
        const p2CardIndices=Array.from(await GameCard.getAllCards(p2.address)).slice(0,deckSize);
        await expect(GameMock.connect(p2).joinMatch(matchIndex,{x: keys[1].pkxy[0], y: keys[1].pkxy[1]},p2CardIndices ))
            .to.revertedWithCustomError(GameMock,"WaitingForFirstPlayerDeck");
        
        await GameMock.connect(p1).setCreatorDeck(matchIndex,p1CardIndices);

        await expect(GameMock.connect(p2).joinMatch(matchIndex,{x: keys[1].pkxy[0], y: keys[1].pkxy[1]},p1CardIndices))
            .to.revertedWithCustomError(GameMock,"CardOwnerMismatch");

        await GameMock.connect(p2).joinMatch(matchIndex,{x: keys[1].pkxy[0], y: keys[1].pkxy[1]},p2CardIndices);
        expect((await GameMock.matches(matchIndex)).state).eq(1);
        //SE
        const gameKeyHex= await GameMock.matches(matchIndex)
        .then(match => match.gameKey.map(bn => hre.ethers.toBeHex(bn)));
        const gameKey = await GameMock.matches(matchIndex)
                          .then(match => match.gameKey.map(bn => hre.ethers.toBeHex(bn)))
                          .then(SE.public_compress);
        
        //console.log("gameKeyHex",gameKeyHex);
        const aggregate_key=SE.aggregate_keys([keys[0].pk,keys[1].pk]);
        console.log("aggregate_key",aggregate_key,gameKey);
        //point="280d653e895291ed1ebe5254700327d321133e4e2e37e53048d7ed21f432e81c" "280d653e895291ed1ebe5254700327d321133e4e2e37e53048d7ed21f432e81c"
        SE.init_prover_key(deckSize)
        const pkc = SE.refresh_joint_key(gameKey, deckSize)
        
        // const match= await GameMock.matches(matchIndex);
        await GameMock.setJointKey(matchIndex,pkc);
        expect((await GameMock.matches(matchIndex)).state).eq(2);
        expect(await GameMock.getPKC(matchIndex)).have.lengthOf(24);
        // expect((await game.duel()).player2Deck).have.lengthOf(0)
        await expect(submitSelfDeck({ game: GameMock, wallet: p1, gameKey, deckSize ,matchIndex}))
        .to.revertedWithCustomError(GameMock,"InvalidState");

        
        //Shuffle env deck
        expect((await GameMock.matches(matchIndex)).envDeck.cards).to.have.length(0);
        await submitEnvInitDeck({ game: GameMock, wallet: p1, gameKey, deckSize ,matchIndex});
        expect((await GameMock.matches(matchIndex)).envDeck.cards).to.have.length(deckSize);
        await submitEnvShuffleDeck({ game: GameMock, wallet: p2, gameKey, 
            deck: (await GameMock.matches(matchIndex)).envDeck.cards ,matchIndex});
        
        expect((await GameMock.matches(matchIndex)).state).eq(3);

        // console.log("shuffleCount",(await GameMock.matches(matchIndex)).envDeck.shuffleCount);
        //Shuffle self deck
        expect((await GameMock.getPlayerData(matchIndex,p1.address)).playerDeck).to.have.lengthOf(0)
        expect((await GameMock.getPlayerData(matchIndex,p2.address)).playerDeck).to.have.lengthOf(0)
        await Promise.all([
            { game: GameMock, wallet: p1, gameKey, deckSize ,matchIndex},
            { game: GameMock, wallet: p2, gameKey, deckSize,matchIndex },
            ].map(submitSelfDeck))
        expect((await GameMock.getPlayerData(matchIndex,p1.address)).playerDeck).to.have.lengthOf(deckSize)
        expect((await GameMock.getPlayerData(matchIndex,p2.address)).playerDeck).to.have.lengthOf(deckSize)
    
        expect((await GameMock.matches(matchIndex)).state).eq(4);
        
        //Shuffle others deck
        await Promise.all([
            { game: GameMock, wallet: p1, gameKey, matchIndex,otherPlayer: p2.address
                ,deck: (await GameMock.getPlayerData(matchIndex,p2.address)).playerDeck },
            { game: GameMock, wallet: p2, gameKey, matchIndex,otherPlayer: p1.address
                ,deck: (await GameMock.getPlayerData(matchIndex,p1.address)).playerDeck },
            ].map(shuffleOpponentDeck))
        
        expect((await GameMock.matches(matchIndex)).state).eq(5);

        SE.init_reveal_key();

        
        //let card=SE.reveal_card_with_snark("0x020b31a672b203b71241031c8ea5e5a4ef133c57bcde822ac514e8a1c7f89124",["0x0bbb65c1461f6b6622f4fcc71f24eca08df3789e4318c1d1f23628a73839d852", "0x2bac4f082c8e1482be425cc89eaf2d347b51aded2901937e6e7bfd0131b14ee2", "0x0139327aac5ec9067c9509587200e581a7b86c3a0338607b62ee8853bf2ee48f", "0x20057633ca7fab6c6834ec0d5bf96f8149c061027ecf0dee6c81777a0076c3a9"]);
        //console.log("test card ",card);

        //Reveal Env Card
        await Promise.all([{game:GameMock, wallet: p1,  matchIndex, sk:keys[0].sk,
            cardIndex: (await GameMock.matches(matchIndex)).envDeck.envRevealIndex,
            deck:(await GameMock.matches(matchIndex)).envDeck.cards },
            {game:GameMock, wallet: p2,  matchIndex,sk:keys[1].sk,
                cardIndex: (await GameMock.matches(matchIndex)).envDeck.envRevealIndex,
                deck:(await GameMock.matches(matchIndex)).envDeck.cards
            }].map(reveaEnvCard));

        expect((await GameMock.matches(matchIndex)).state).eq(6);
        
        const matchEnvCards=await GameMock.getMatchEnvCards(matchIndex);
        console.log("matchEnvCards", matchEnvCards.map((m)=>m.cardType).join(","));
        const boardCard= matchEnvCards[+(await GameMock.matches(matchIndex)).envDeck.envBoard.toString()];
        console.log("revealed board card",boardCard);

        //
        await Promise.all([{game: GameMock, sk: keys[0].sk,
            revealCount:3, revealIndex:0, matchIndex:matchIndex, 
            playerIndex:1, deck: (await GameMock.getPlayerData(matchIndex,p2.address)).playerDeck,
            wallet: p1,
        },
        {game: GameMock, sk: keys[1].sk,
            revealCount:3, revealIndex:0, matchIndex:matchIndex, 
            playerIndex:0, deck: (await GameMock.getPlayerData(matchIndex,p1.address)).playerDeck,
            wallet: p2,
        }
        ].map(showOpponentHand))
        
        expect((await GameMock.matches(matchIndex)).state).eq(7);

        let player1Data=(await GameMock.getPlayerData(matchIndex,p1.address));
        let player2Data=(await GameMock.getPlayerData(matchIndex,p2.address));
        expect(player1Data.playerHand).have.lengthOf(3);

        const [player1NftIds,player2NftIds]=await Promise.all([
            { deck: player1Data.playerDeck, 
                sk: keys[0].sk, hands: player1Data.playerHand.map(m=>+m.toString(10)), 
                rTokens: player1Data.playerReveals, nfts: player1Data.originalCards, name: 'Player 1' },
            {deck: player2Data.playerDeck, 
                sk: keys[1].sk, hands: player2Data.playerHand.map(m=>+m.toString(10)), 
                rTokens: player2Data.playerReveals, nfts: player2Data.originalCards, name: 'Player 2'},
            ].map(printHandCards))
        console.log(`'Player 1: ${player1NftIds.join(' ')}`)
        console.log(`'Player 2: ${player2NftIds.join(' ')}`)
        
        //Play card 
        let player1PlayCard=2;
        await playCardTxn({game: GameMock, wallet:p1, sk: keys[0].sk, hand: player1Data.playerHand.map(m=>+m.toString(10)),
            handIndex:player1PlayCard, deck: player1Data.playerDeck, matchIndex
        })

        expect((await GameMock.getPlayerData(matchIndex,p1.address)).playerBoard).eq(player1NftIds[player1PlayCard]);

        let player2PlayCard=1;
        await playCardTxn({game: GameMock, wallet:p2, sk: keys[1].sk, hand: player2Data.playerHand.map(m=>+m.toString(10)),
            handIndex:player2PlayCard, deck: player2Data.playerDeck, matchIndex
        })

        //Round Finish Now should be player2
        expect((await GameMock.matches(matchIndex)).playerTurn).eq(1);
        expect((await GameMock.getPlayerData(matchIndex,p2.address)).playerBoard).eq(player2NftIds[player2PlayCard]);
        expect((await GameMock.matches(matchIndex)).rounds).eq(1);
        //Next round
        expect((await GameMock.getPlayerData(matchIndex,p1.address)).playerRevealCount).eq(3);
        expect((await GameMock.getPlayerData(matchIndex,p2.address)).playerRevealCount).eq(3);
        
        //Reveal player2 card
        player1Data=(await GameMock.getPlayerData(matchIndex,p1.address));
        player2Data=(await GameMock.getPlayerData(matchIndex,p2.address));
        await expect(showOpponentHand({game: GameMock, sk: keys[1].sk,
            revealCount:2, revealIndex:0, matchIndex:matchIndex, 
            playerIndex:1, deck: player1Data.playerDeck,
            wallet: p2,
        })).to.revertedWithCustomError(GameMock,"CardRevealCountError");
        
        await expect(showOpponentHand({game: GameMock, sk: keys[1].sk,
            revealCount:1, revealIndex:1, matchIndex:matchIndex, 
            playerIndex:0, deck: player1Data.playerDeck,
            wallet: p1,
        })).to.revertedWithCustomError(GameMock,"CannotShowPlayerCards");

        await GameMock.connect(p2).playerAction(matchIndex,0);
     
        await showOpponentHand({game: GameMock, sk: keys[0].sk,
            revealCount:1, revealIndex:+player2Data.playerRevealCount.toString(), matchIndex:matchIndex, 
            playerIndex:1, deck: player2Data.playerDeck,
            wallet: p1,
        })
        player2Data=(await GameMock.getPlayerData(matchIndex,p2.address));
        expect(player2Data.playerRevealCount).eq(4);
        expect(player2Data.playerHand).have.lengthOf(3);
        console.log("player2Data.playerHand",player2Data.playerHand);
        player2PlayCard=2;
        //Play player2 card
        await expect(playCardTxn({game: GameMock, wallet:p1, sk: keys[0].sk, hand: player2Data.playerHand.map(m=>+m.toString(10)),
            handIndex:player2PlayCard, deck: player2Data.playerDeck, matchIndex
        })).to.revertedWithCustomError(GameMock, "PlayerTurnError");
        await playCardTxn({game: GameMock, wallet:p2, sk: keys[1].sk, hand: player2Data.playerHand.map(m=>+m.toString(10)),
            handIndex:player2PlayCard, deck: player2Data.playerDeck, matchIndex
        })
        //Reveal player1 card
        
         //Play player1 card
        await playCard({matchIndex,handIndex:1,keys,playerIndex:0,
            players,revealCount:1
        })
        //Next round
        //Round Finish Now should be player2
        expect((await GameMock.matches(matchIndex)).playerTurn).eq(0);
        expect((await GameMock.matches(matchIndex)).rounds).eq(2);
        expect((await GameMock.getPlayerData(matchIndex,p1.address)).playerRevealCount).eq(4);
        expect((await GameMock.getPlayerData(matchIndex,p2.address)).playerRevealCount).eq(4);
        
        player1Data=(await GameMock.getPlayerData(matchIndex,p1.address));
        player2Data=(await GameMock.getPlayerData(matchIndex,p2.address));
        console.log("Round 2 score p1=",player1Data.position,",p2=",player2Data.position);
        
        // await GameMock.connect(p1).playerAction(matchIndex,1);
        // //Reveal Env Card(Player 1 turn)
        // player1Data=(await GameMock.getPlayerData(matchIndex,p1.address));
        // await expect(showOpponentHand({game: GameMock, sk: keys[1].sk,
        //     revealCount:1, revealIndex:+player1Data.playerRevealCount.toString(), matchIndex:matchIndex, 
        //     playerIndex:0, deck: player1Data.playerDeck,
        //     wallet: p2,
        // })).to.revertedWithCustomError(GameMock,"CannotShowPlayerCards");
 
        for(var i=0;i<10;i++){
            await playTurn({keys,matchIndex,players,revealEnv:i%3});
            const matchData=await GameMock.matches(matchIndex);
            if(matchData.state==8n){
                break;
            }
        }

        await expect(
            VrfCoordinatorV2Mock.fulfillRandomWords(4, (await VRF.getAddress()))
        ).to.emit(VrfCoordinatorV2Mock, "RandomWordsFulfilled").withArgs(4,anyValue,anyValue,true);
        expect(await GameCard.getAllCards((await GameMock.matches(matchIndex)).winner)).have.lengthOf(nftCount+1n);
        
        expect((await GameMock.matches(matchIndex)).winnersCard).eq(await GameCard.getLatestCard((await GameMock.matches(matchIndex)).winner));
    })
})