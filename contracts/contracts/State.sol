// SPDX-License-Identifier: GPL-3.0
import { Point} from '@zypher-game/secret-engine/Verifiers.sol';
pragma solidity ^0.8.0;
library IState{
    
    uint32 public constant PKC_SIZE=24;
    
    enum CardRarity{
        S, //2
        A, //8
        B,  //15
        C,  //25
        D   //50
    }
    //
    // enum RequestType{
    //     Player,
    //     Env
    // }


    enum MatchState{
        None,
        //InitEnvDeck,
        SetPkc,
        ShuffleEnvDeck,
        SubmitSelfDeck,
        ShuffleOpponentDeck,
        RevealEnvCard,
        RevealPlayersHand,
        PlayerPlayCard,
        Finished,
        __UNUSED_7__
    }
 
    struct Match{
        IState.MatchState state;
        mapping(uint8=>PlayerMatchData) players;
        uint8 playerCount;
        Point gameKey;
        uint256[PKC_SIZE] pkc;
        uint8 playerTurn;
        uint8 playerTurnType;
        uint8 revealEnv;
        // uint8 isFinished;
        uint8 turnStart;
        uint8 rounds;
        address winner;
        EnvDeck envDeck;
        address creator;
        uint256 winnersCard;
        // DiscardDeck discardDeck;
        // uint8 orderReverse;
    }

    struct EnvDeck{
        //Total deck 
        // uint256[] originalCards;
        uint256[4][] cards;
        uint256[2][][] envReveals;
        uint8 envRevealIndex;
        uint256 envBoard;
        uint256 shuffleCount;
        uint256 playerRevealProofIndex;
    }

   

    struct PlayerMatchData{
        uint256[] originalCards;
        address player;
        bool done;
        Point publicKey;
        uint8 playerIndex;
        uint256[4][] playerDeck;
        //Can reach total deck size
        uint256[2][][] playerReveals;
        uint8 playerRevealCount;
        uint256 playerRevealProofIndex;
        //Can be upto TOTAL_HAND_SIZE
        uint8[] playerHand;
        uint256 playerHandIndex;
        //Only one active card on board
        uint256 playerBoard;
        uint256 position;
        uint256 nextRoundPlayerRevealCount;
    }

    function getRarityOld(uint256 randomRarity,uint256 minRarity) public pure returns(CardRarity){
        uint256 randomNumber=0;
        if(minRarity>uint256(CardRarity.C)){
            return CardRarity.D;
        }else if(minRarity>uint256(CardRarity.B)){
            randomNumber = (randomRarity % 75) +25;
        }else if(minRarity>uint256(CardRarity.A)){
            randomNumber = (randomRarity % 90) +10;
        }else if(minRarity>uint256(CardRarity.S)){
            randomNumber = (randomRarity % 98) +2;
        }else{
            randomNumber = (randomRarity % 100);
        }

        if(randomNumber>=50){
            return CardRarity.D;
        }else if(randomNumber>=25){
            return CardRarity.C;
        }else if(randomNumber>=10){
            return CardRarity.B;
        }else if(randomNumber>=2){
            return CardRarity.A;
        }else{
            return CardRarity.S;
        }
    }

    function getRarity(uint256 randomRarity,uint256 minRarity) public pure returns(uint8){
        uint256 randomNumber=0;
        if(minRarity>uint256(CardRarity.C)){
            return uint8(CardRarity.D);
        }else if(minRarity>uint256(CardRarity.B)){
            randomNumber = (randomRarity % 75) +25;
        }else if(minRarity>uint256(CardRarity.A)){
            randomNumber = (randomRarity % 90) +10;
        }else if(minRarity>uint256(CardRarity.S)){
            randomNumber = (randomRarity % 98) +2;
        }else{
            randomNumber = (randomRarity % 100);
        }

        if(randomNumber>=50){
            return uint8(CardRarity.D);
        }else if(randomNumber>=25){
            return uint8(CardRarity.C);
        }else if(randomNumber>=10){
            return uint8(CardRarity.B);
        }else if(randomNumber>=2){
            return uint8(CardRarity.A);
        }else{
            return uint8(CardRarity.S);
        }
    }

    function createCard(uint256 randomRarity, uint256 randomCard, uint8 minRarity) public pure returns(AnimalCard memory){
        CardRarity rarity = CardRarity(getRarity(randomRarity,uint256(minRarity)));
        AnimalCard memory card =getAnimal(randomCard,rarity);
        return card;
    }

    function getEnvCard(uint256 randomCardType,uint256 randomCardValue) public pure returns(EnvCard memory){
        uint256 limit =uint256(EnvCardType.Enemy)+1;
        uint8 t=uint8(randomCardType % limit);
        EnvCardType val= EnvCardType(t);
        uint8 card=getEnvValue(val, randomCardValue);
        EnvCard memory envCard= EnvCard({cardType: val,card: card});
        return envCard;
    }

    function getEnvValue(EnvCardType val,uint256 randomCardValue) private pure returns(uint8){
        if(val==EnvCardType.Geography){
            return uint8(randomCardValue%(uint256(Geography.G6)+1));
        }
        return uint8(randomCardValue%(uint256(Enemy.E3)+1));
    }

    //6
    enum Geography{
        G1,//Forest,
        G2,//Mountain,
        G3,//Swamp,
        G4,//Desert,
        G5,//River,
        G6//Grass
    }

    //3
    enum Enemy{
        E1,//Tiger,
        E2,//Crocodile,
        E3//Bear
    }

    enum EnvCardType{
        Geography,
        Enemy
    }

    struct EnvCard{
        EnvCardType cardType;
        uint8 card;
    }

    struct AnimalCard{
       
        uint256 prop;
    }

    enum Animal{
        S1,
        S2,
        A1,
        A2,
        B1,
        B2,
        C1,
        C2,
        D1,
        D2
    }
    //S
    //Horse 
    //A
    //Snake,Deer, Monkey
    //B
    //Tortoise,Rabbit,Dog,Cat,Duck
    //C
    //Goat,Sheep
    //D
    //Snail,Frog,

    function getAnimal(uint256 randomCard,CardRarity rarity) public pure returns (AnimalCard memory card){
        // (AnimalCard[] memory cards,uint256[] memory raritySizes)=getAllAnimalCards();
        // uint256 rarityNum = uint256(rarity);
        // uint256 randomNum=randomCard%raritySizes[rarityNum];
        // card= cards[randomNum*2+randomNum];
        card = getRandomizedAnimalCard(randomCard,rarity);
        return card;
    }

    function getRandomizedAnimalCard(uint256 randomCard,CardRarity rarity) public pure returns(AnimalCard memory card)
    {
        //8+8+3+2
        uint8 geographies=uint8(randomCard & 63);
        uint8 enemies= uint8(randomCard >> 8 & 7);
        uint8 health = uint8(randomCard >> 3 & 7);
        uint8 animal = uint8(randomCard >> 3 & 1);

        uint8 steps=5-uint8(rarity);
        card = AnimalCard({
            prop: getAnimalProp(rarity,Animal(animal+2*uint8(rarity)),health,enemies,geographies,steps)
        });
        return card;
    } 

    //3 + 6 + 3 + 8 + 8 + 3
    function getAnimalProp(CardRarity rarity, 
    Animal animal, uint8 health, uint8 weakness, uint8 favoredGeographies, uint8 steps)
    public pure returns(uint256)
    {
        uint256 val=0;
        val=val+uint8(rarity);
        val=val<<6;
        val=val+uint8(animal);
        val=val<<3;
        val=val+health;
        val=val<<8;
        val=val+weakness;
        val=val<<8;
        val=val+favoredGeographies;
        val=val<<3;
        val=val+steps;
        return val;
        
    }

    function getAnimalRarity(uint256 prop) public pure returns(uint8) {
        return uint8(prop>>28 & 7);
    }

    function getAnimalSteps(uint256 prop) internal pure returns(uint8) {
        return uint8(prop & 7);
    }

    function getAnimalWeakness(uint256 prop) internal pure returns(uint8) {
        return uint8((prop>>3 + 8) & 127);
    }

    function calculateAnimalScore(AnimalCard memory animal, EnvCard memory env) private pure returns(uint256){
        EnvCardType eType=EnvCardType(env.cardType);
        uint8 steps=getAnimalSteps(animal.prop);
        uint8 weakness=getAnimalSteps(animal.prop);
        if(eType == EnvCardType.Enemy){
            if(weakness>>env.card & 1 == 1){
                if (steps>=2) {
                    steps-=2;
                }else{
                    steps=0;
                }
            }
        }else if(eType == EnvCardType.Geography){
            steps+=1;
        }   
        if(steps<0){
            steps=0;
        }
        return steps;
    }

    function calculateRoundScore(AnimalCard[] memory playerCards,EnvCard memory  envCard )
     public pure returns(uint256[] memory){
        uint256[] memory scores=new uint256[](playerCards.length);
        for(uint8 i =0;i<playerCards.length;i++){
            scores[i]=calculateAnimalScore( playerCards[i],envCard );
        }
        return scores;
    }

    
}