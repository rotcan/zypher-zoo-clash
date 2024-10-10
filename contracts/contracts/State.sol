// SPDX-License-Identifier: GPL-3.0
import { Point} from '@zypher-game/secret-engine/Verifiers.sol';
pragma solidity ^0.8.0;
library IState{
    

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
        uint8 t=uint8(randomCardType % (uint256(EnvCardType.Enemy)));
        EnvCardType val= EnvCardType(t);
        uint8 card=getEnvValue(val, randomCardValue);
        EnvCard memory envCard= EnvCard({cardType: val,card: card});
        return envCard;
    }

    function getEnvValue(EnvCardType val,uint256 randomCardValue) private pure returns(uint8){
        if(val==EnvCardType.Geography){
            return uint8(randomCardValue%(uint256(Geography.G6)));
        }
        return uint8(randomCardValue%(uint256(Enemy.E3)));
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
        // CardRarity rarity;
        // Animal animal;
        // //uint shield;
        // uint health;
        // uint weakness;
        // uint favoredGeographies;
        // uint steps;
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

    // function getAllAnimalCards() public pure returns(AnimalCard[] memory cards, uint256[] memory raritySizes){
    //     cards=new AnimalCard[](10);
    //     raritySizes=new uint256[](5);
    //     uint counter=0;
    //     cards[counter++]=AnimalCard({
    //         prop: getAnimalProp(CardRarity.S,Animal.S1,2,5,37,5)
    //     });
    //     cards[counter++]=AnimalCard({
    //         prop: getAnimalProp(CardRarity.S,Animal.S2,2,5,37,5)
    //     });
        
    //     cards[counter++]=AnimalCard({
    //        prop: getAnimalProp(CardRarity.A,Animal.A1,3,7,37,4)
    //     });

    //     cards[counter++]=AnimalCard({
    //         prop: getAnimalProp(CardRarity.A,Animal.A2,2,0,37,4)
    //     });

    //     cards[counter++]=AnimalCard({
    //          prop: getAnimalProp(CardRarity.B,Animal.B1,2,5,37,3)
    //     });

    //     cards[counter++]=AnimalCard({
    //         prop: getAnimalProp(CardRarity.B,Animal.B2,2,5,37,3)
    //     });

    //     cards[counter++]=AnimalCard({
    //         // rarity: CardRarity.C,
    //         // animal: Animal.C1,
    //         // // shield: 1,
    //         // health: 2,
    //         // weakness: 5, //101
    //         // favoredGeographies: 37,//010101 
    //         // steps:1
    //         prop: getAnimalProp(CardRarity.C,Animal.C1,2,5,37,2)
    //     });
    
    //     cards[counter++]=AnimalCard({
    //         // rarity: CardRarity.C,
    //         // animal: Animal.C2,
    //         // // shield: 1,
    //         // health: 2,
    //         // weakness: 5, //101
    //         // favoredGeographies: 37,//010101 
    //         // steps:1
    //         prop: getAnimalProp(CardRarity.C,Animal.C2,2,5,37,2)
    //     });

    //     cards[counter++]=AnimalCard({
    //         // rarity: CardRarity.D,
    //         // animal: Animal.D1,
    //         // // shield: 1,
    //         // health: 2,
    //         // weakness: 5, //101
    //         // favoredGeographies: 37,//010101 
    //         // steps:1
    //         prop: getAnimalProp(CardRarity.D,Animal.D1,2,5,37,1)
    //     });

    //     cards[counter++]=AnimalCard({
    //         // rarity: CardRarity.D,
    //         // animal: Animal.D2,
    //         // // shield: 1,
    //         // health: 2,
    //         // weakness: 5, //101
    //         // favoredGeographies: 37,//010101 
    //         // steps:1
    //         prop: getAnimalProp(CardRarity.D,Animal.D2,2,5,37,1)
    //     });

    //     raritySizes[uint256(CardRarity.S)]=2;
    //     raritySizes[uint256(CardRarity.A)]=2;
    //     raritySizes[uint256(CardRarity.B)]=2;
    //     raritySizes[uint256(CardRarity.C)]=2;
    //     raritySizes[uint256(CardRarity.D)]=2;
    //     return (cards,raritySizes);
    // }

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
                steps-=2;
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

    //-----------------PRIVATE VALUES----------------

    function getCardIndex(Point memory unmaskPoint) public pure returns (uint){
        uint256[54] memory cards_map = [
        0x0e7e20b3cb30785b64cd6972e2ddf919db64d03d6cf01456243c5ef2fb766a65,
        0x2d7690deeaa77c9d89b0ceb3c25f7bb09c44f40b4b8cf5d6fcb512c7be8fcba9,
        0x13a50334ef174fd8160bb22e5f150b0ce7656c5c4a19b0ad6bc8f93fdf5fab7c,
        0x02acd55fbf59ea2b7a4733ccb5568681e6445d2cba2a4ee0707c1c1d3bc27fea,
        0x17fd6b5a880d0570dad7bd4da582c2ba03717615764e3955a8bf2a1b546abfa2,
        0x10b37010cd0d430a2bc91ee19f30d1a3d5984605dc299953fdd1ef2fff2f1a95,
        0x2a6a6ec33c00e9d9073ce5e48f45afd40cb29303bbc0367606c6f2963ec057c9,
        0x27bfe4a93f3e0802f37732ef692a7ff681ce6baaacb6e1cc73e972374e58cec2,
        0x2627f2b312c0f1f30b638a1ccc76c7025e94d99cc6006229432fa431044cf7aa,
        0x0eb99c13f783f3416210d34a8e5fa766ae239c4c00cb9d3e81f14dc975a7a957,
        0x1245109a40dc41351a708f1b7c6fb8bcf809c656b366fb1d0fa7a46991d2b977,
        0x000f90cf5f6433978210b9098c0e0865d44f6bc4ab9a7c3cfa63ed7e586f8fa7,
        0x2c957cd805d207f518047f6117ecd42fa98b78734efe4cb588cd409ff25aa0b8,
        0x2d4b20b261ace4d99d8d80a0998133b0f5c49bad68a4a9a92e9fe2084c8dcde8,
        0x23f5c25e039914df2928a715bf68c41ba91b51103d1b1aeaba9323b677b9ea8d,
        0x04578915cb17f8fd142120c1bd5c0a26da6668cd746aad9ce707ccfd4464533f,
        0x18d33bc856f163194090c1c6419aedbedfaf6dcfb23588ce7002d7deb6ea7623,
        0x1db8329a5d644ab56185ebb02724b836c5b1d22d29a57965a0e3a43067e06a08,
        0x17a87862cbcee70b0cd0c442d36e26ed763385bb2e948d8f00469d908aa07e72,
        0x13fa0efab13db7078ee0aa83cf8fd476614c779e530da57c2101177e69cd68e3,
        0x16d52c3e7be3ab38454acdfa2cd7a3cb7a321092f41f038a3ae4f1947bad724e,
        0x14157ff39b00904e49f284a3ae75e225b995e3b123887c2ddea019e791fcf88d,
        0x0967dd7bac9eb504b37cf33860d77e8ed747f54864aabb63b2487c3f249dd2d2,
        0x0047239fd59b5ce078d0fa8a1f0c667b2355fb331bfcfe5fe58754cdade49f2b,
        0x0f220815394d328c3a819ca5dc13219b422b8443eca0b8e6911d2b0078d1bb68,
        0x04c1f519b090dac2ebff9282ca66592f8b9b6c8c2e38705740daa1230fe2b6cc,
        0x169a776c4976ebb48f3c2f3eb6214f26ac70557acd6a28c95044653dee7c7306,
        0x17859495fda1f3ac4d240997cfa7d61d9624006410ddc97c7060a24e9fc1053a,
        0x250f584b0539ef28cb0b7a136b26a2b796fbbde5a0df8236b4775c0e713ef8c8,
        0x025761ba480df2787230ecd283209f959b80a16ff631b751e2213a431a0be30c,
        0x0ac3e3209fa174e4981b53a69ce6c5cbca1e217262a27826621553d15fce1317,
        0x1daa7bc5da2abf17ed3a43a4a3ddec8e0ed6cc3f2a729b6bfab7f4f252f47197,
        0x17e97bb5c68c80f4c0f38eebf4106b0c8ec02c6d9d678588be5f4a71b43c86fe,
        0x1dcedb86bb03fa3b404afd3edaa59ceaf8122b2e9dc35c1cdc9f4c65ac6df154,
        0x2f2ce3a1cddb1e92541481d30b7c43af5d0350266672632ad06728818b6affdb,
        0x2c9fb046ab1f36b104b456598d00e3211fb31b0ef357d7c7de55c4a122257dbe,
        0x078d7b6afe9372d90a9b9e2e5f40dc97c06bed7821c0870c8f19847cb4d6d5ce,
        0x0548073474086bb9f2f2eda49f8625572f2be9d6b71bb293388e3ff9ad8fb7aa,
        0x012b6918773feaa8a22ac16c2e243f2c371c98dbf13801ad0bb9f4cee4575c8d,
        0x1abcecb5d562b19da37897d7db6f6227be857493300e1f38d234b43d36037b5d,
        0x2fb979bcc2cc562386634c502b9425003d9c0876250b28e21996de4babe104cc,
        0x173e80227d906db5ba7289d3611dae189797aa8e3e235949e76d2ce97f6f3c73,
        0x022a95649ff5d46713821806b85466cf709ad85171567cf1c0692940793dd30f,
        0x00fbc18c6483aef1404ac3e81cae370bb7c9548b5d76124017d522043fc19a6c,
        0x1d65fcc3af60454fcb4b6a5fd74eb5c3305757a8a47ff7d07cd92e74cb2a1fbb,
        0x227532d0e59b89a139600b60e96a3a8950a93dfa61e40ae623bd16f5529c0687,
        0x10f119e93c8adb81acde0c8876e199a30fa0e5f96345a14ab5e6aee59ad80e12,
        0x1785b53f50e8bb17af2e5394c3c12bcf8349c13b45a0f0aff2da29070e2109b2,
        0x0e928ce03f8f6d07a6c818b295bbc453034e07c55f43eb85b576b22739eb4a51,
        0x0d55d8fae5d67f985fe733eaf647f53f42490c2226e54bb7058031fc5e4ef58e,
        0x0759e62cf2464671501c16a8534d28bc2e5721a1de966ff2ef9e924424765f41,
        0x25bebd6ecfef4f2613efc455e4038489febf84079c88c787977fee2e07629b4b,
        0x1464429b0e93a259cec0b660c0bb6df28cb408706eee28f4e77a5e61c931f6f5,
        0x142fb87f6d0974097206facac23ea38ffa01e3e1e45003e3ec238b6516eb0b2e
    ];

        for(uint i=0;i<cards_map.length;i++){
            if(unmaskPoint.y==cards_map[i]){
                return i;
            }
        }
        return 0;
    }
}