// SPDX-License-Identifier: GPL-3.0

pragma solidity ^0.8.20;
import {IGameCard} from "./GameCard.sol";
import {ZgShuffleVerifier, ZgRevealVerifier, Point, MaskedCard} from '@zypher-game/secret-engine/Verifiers.sol';
import {ShuffleService} from "./shuffle/ShuffleService.sol";



contract Game {
    IGameCard public cardSet;
    uint256 private constant VALID_DECK_SIZE = 20;
    

    
    mapping(address => uint256[]) private _decks;
    mapping(uint256=>address) private _winners;

    ShuffleService private _shuffleService;
    ZgRevealVerifier private _reveal;

    constructor(address _card) {
        cardSet = IGameCard(_card);
        _initDuel();
    }


    function getDeck(address deckOwner) external view returns (uint256[] memory) {
        return _decks[deckOwner];
    }

    function setVerifiers(ShuffleService shuffle, ZgRevealVerifier reveal) external {
        _shuffleService=shuffle;
        _reveal = reveal;
    }

    error DuplicateCard(uint256 cardId);
    error CardNotFound(uint256 cardId);
    uint256 private constant CARD_NOT_FOUND=type(uint256).max;

    function addCard(uint256 cardId) external {
        if (_findCardIndex(msg.sender, cardId) != CARD_NOT_FOUND) {
            revert DuplicateCard(cardId);
        }

        _decks[msg.sender].push(cardId);
    }

    function removeCard(uint256 cardId) external {
        uint256[] storage deck = _decks[msg.sender];
        uint256 index = _findCardIndex(msg.sender, cardId);

        if (index >= deck.length) {
            revert CardNotFound(cardId);
        }

        deck[index] = deck[deck.length - 1];
        deck.pop();
    }


    function _findCardIndex(
        address deckOwner,
        uint256 cardId
    ) internal view returns (uint256) {
        uint256[] storage deck = _decks[deckOwner];

        for (uint256 i = 0; i < deck.length; i++) {
            if (deck[i] == cardId) {
                return i;
            }
        }

        return CARD_NOT_FOUND;
    }

    function isValidDeck(address deckOwner) public view returns (bool) {
        uint256[] storage deck = _decks[deckOwner];

        if (deck.length != VALID_DECK_SIZE) {
            return false;
        }

        for (uint256 i = 0; i < deck.length; i++) {
            if (cardSet.ownerOf(deck[i]) != deckOwner) {
                return false;
            }
        }

        return true;
    }


    enum DuelState{
        None,
        SubmitSelfDeck,
        ShuffleOpponentDeck,
        RevealInitialHand,
        Player1_Round,
        Player2_Round,
        Finished,
        __UNUSED_7__
    }

    struct Duel{
        DuelState state;
        address player1;
        address player2;
        bool done1;
        bool done2;
        Point publicKey1;
        Point publicKey2;
        Point gameKey;
        uint256[] pkc;
        uint256[4][] player1Deck;
        uint256[4][] player2Deck;
        uint256[2][][] player1Reveals;
        uint256[2][][] player2Reveals;
        uint8[] player1Hand;
        uint8[] player2Hand;
        uint256[] player1Board;
        uint256[] player2Board;
        uint8 player1Lives;
        uint8 player2Lives;
    }

    uint256 private _duelId;
    Duel private _duel;
    event DuelFinished(uint256 duelId,address winner);

    function duel() external view returns (Duel memory){
        return _duel;
    }

    event DuelReset();

    uint8 private constant INIT_HAND_SIZE = 3;
    uint8 private constant PLAYER_LIVES = 5;

    function newDuel() external {
        delete _duel;
        _initDuel();

        emit DuelReset();
    }


    function _nextDuelState() internal returns (DuelState) {
        _duel.state = DuelState(uint256(_duel.state) + 1);
        _duel.done1=false;
        _duel.done2=false;
        return _duel.state;
    }

    function _initDuel() internal {
        //[Reveal tokens][No of players][No of cards]
        _duel.player1Reveals = new uint256[2][][](VALID_DECK_SIZE);
        _duel.player2Reveals = new uint256[2][][](VALID_DECK_SIZE);

        _duel.player1Hand=  new uint8[](INIT_HAND_SIZE);
        _duel.player2Hand=  new uint8[](INIT_HAND_SIZE);

        _duel.player1Lives = PLAYER_LIVES;
        _duel.player2Lives = PLAYER_LIVES;
    }

    error InvalidDeck(address player);
    error InvalidDuelState(DuelState current, DuelState expected);
    error DuelAlreadyFull();
    event DuelStarted(address player1, address player2);
    function joinDuel(Point memory publicKey) external {
        address player = msg.sender;

        if (!isValidDeck(player)) {
            revert InvalidDeck(player);
        }

        if (_duel.state != DuelState.None) {
            revert InvalidDuelState(_duel.state, DuelState.None);
        }

        if (_duel.player1 == address(0)) {
            _duel.player1 = player;
            _duel.publicKey1=publicKey;
        } else if (_duel.player2 == address(0)) {
            _duel.player2 = player;
            _duel.publicKey2=publicKey;

             Point[] memory playerKeys = new Point[](2);
             playerKeys[0]=_duel.publicKey1;
             playerKeys[1]=_duel.publicKey2;
            _duel.gameKey = _reveal.aggregateKeys(playerKeys);

            _nextDuelState();
            emit DuelStarted(_duel.player1, _duel.player2);
        } else {
            revert DuelAlreadyFull();
        }
    }
    

    error InvalidShuffle();
    function submitDeck(
        uint256[24] calldata pkc,
        uint256[4][VALID_DECK_SIZE] calldata maskedDeck,
        uint256[4][VALID_DECK_SIZE] calldata shuffledDeck,
        bytes calldata proof
    ) external {
        // Do some simple checks...

        _duel.pkc = pkc;
        _shuffleService.setPkc(_duel.pkc);
        // uint256[] memory input = new uint256[](VALID_DECK_SIZE * 4 * 2);
        // for (uint256 i = 0; i < VALID_DECK_SIZE; i++) {
        //     input[i * 4 + 0] = maskedDeck[i][0];
        //     input[i * 4 + 1] = maskedDeck[i][1];
        //     input[i * 4 + 2] = maskedDeck[i][2];
        //     input[i * 4 + 3] = maskedDeck[i][3];

        //     input[i * 4 + 0 + VALID_DECK_SIZE*4] = shuffledDeck[i][0];
        //     input[i * 4 + 1 + VALID_DECK_SIZE*4] = shuffledDeck[i][1];
        //     input[i * 4 + 2 + VALID_DECK_SIZE*4] = shuffledDeck[i][2];
        //     input[i * 4 + 3 + VALID_DECK_SIZE*4] = shuffledDeck[i][3];
        // }
        uint256[] memory maskedDeckInput = new uint256[](VALID_DECK_SIZE*4);
        uint256[] memory shuffledDeckInput = new uint256[](VALID_DECK_SIZE*4);
        
        for (uint256 i = 0; i < VALID_DECK_SIZE; i++) {
            maskedDeckInput[i * 4 + 0] = maskedDeck[i][0];
            maskedDeckInput[i * 4 + 1] = maskedDeck[i][1];
            maskedDeckInput[i * 4 + 2] = maskedDeck[i][2];
            maskedDeckInput[i * 4 + 3] = maskedDeck[i][3];

            shuffledDeckInput[i * 4 + 0] = shuffledDeck[i][0];
            shuffledDeckInput[i * 4 + 1] = shuffledDeck[i][1];
            shuffledDeckInput[i * 4 + 2] = shuffledDeck[i][2];
            shuffledDeckInput[i * 4 + 3] = shuffledDeck[i][3];
        }
        _shuffleService.setDeck(maskedDeckInput);
        _shuffleService.verify(shuffledDeckInput, proof);
        //;(proof, input, _duel.pkc)
        // {
        //  revert InvalidShuffle();
        // }

        // Store deck...
        address player = msg.sender;
        if(player == _duel.player1){
            _duel.player1Deck=shuffledDeck;
            _duel.done1 = true;
        }
        else if(player == _duel.player2){
            _duel.player2Deck=shuffledDeck;
            _duel.done2 = true;
        }
        else{
            revert InvalidDeck(player);
        }
        if (_duel.done1 && _duel.done2){
            _nextDuelState();
        }
    }

    function shuffleDeck(
        uint256[4][VALID_DECK_SIZE] calldata shuffledDeck,
        bytes calldata proof
    ) external {
        // Do some simple checks...
        address player = msg.sender;

        if(player !=_duel.player1 && player!=_duel.player2){
             revert InvalidDeck(player);
        }
        uint256[4][] memory currentDeck = player == _duel.player1 ? _duel.player2Deck : _duel.player1Deck;
        
        uint256[] memory maskedDeckInput = new uint256[](VALID_DECK_SIZE*4);
        uint256[] memory shuffledDeckInput = new uint256[](VALID_DECK_SIZE*4);
        for (uint256 i = 0; i < VALID_DECK_SIZE; i++) {
            maskedDeckInput[i * 4 + 0] = currentDeck[i][0];
            maskedDeckInput[i * 4 + 1] = currentDeck[i][1];
            maskedDeckInput[i * 4 + 2] = currentDeck[i][2];
            maskedDeckInput[i * 4 + 3] = currentDeck[i][3];

            shuffledDeckInput[i * 4 + 0] = shuffledDeck[i][0];
            shuffledDeckInput[i * 4 + 1] = shuffledDeck[i][1];
            shuffledDeckInput[i * 4 + 2] = shuffledDeck[i][2];
            shuffledDeckInput[i * 4 + 3] = shuffledDeck[i][3];
        }
        //Set current deck
        _shuffleService.setDeck(maskedDeckInput);
        //Set and verify shuffled deck
        _shuffleService.verify(shuffledDeckInput, proof);
        //Store deck
        if(player == _duel.player1){
            _duel.player2Deck=shuffledDeck;
            _duel.done1 = true;
        }
        else {
            _duel.player1Deck=shuffledDeck;
            _duel.done2 = true;
        }
        if (_duel.done1 && _duel.done2){
            for(uint8 i=0;i<INIT_HAND_SIZE;i++){
                _duel.player1Hand[i]=i;
                _duel.player2Hand[i]=i;
            }
            _nextDuelState();
        }

    }

    error InvalidRevealToken();
    function _requireValidRevealToken(
        uint8 targetPlayerId,
        uint8 targetCardIndex,
        uint8 senderPlayerId,
        uint256[2] calldata revealToken,
        uint256[8] calldata proof
    ) internal view {
        uint256[4][] storage deck = targetPlayerId == 1 ? _duel.player1Deck : _duel.player2Deck;
        Point storage publicKey = senderPlayerId == 1 ? _duel.publicKey1 : _duel.publicKey2;

        if (!_reveal.verifyRevealWithSnark([
            deck[targetCardIndex][2],
            deck[targetCardIndex][3],
            revealToken[0],
            revealToken[1],
            publicKey.x,
            publicKey.y
        ], proof)) {
            revert InvalidRevealToken();
        }
    }

    function _realCardId(
        uint8 playerId,
        uint8 cardIndex
    ) internal view returns (uint256) {
        address player = playerId == 1 ? _duel.player1 : _duel.player2;

        uint256[4] storage maskedCard = playerId == 1 ? _duel.player1Deck[cardIndex] : _duel.player2Deck[cardIndex];
        uint256[2][] storage reveals = playerId == 1 ? _duel.player1Reveals[cardIndex] : _duel.player2Reveals[cardIndex];

        if (reveals.length < 2) {
            return 0;
        }

        Point[] memory rTokens = new Point[](reveals.length);
        for (uint256 i = 0; i < reveals.length; i++) {
            rTokens[i] = Point(reveals[i][0], reveals[i][1]);
        }

        Point memory realCardPoint = _reveal.unmask(MaskedCard(
            maskedCard[0],
            maskedCard[1],
            maskedCard[2],
            maskedCard[3]
        ), rTokens);

        uint realCardIndex=getCardIndex(realCardPoint);
        return _decks[player][realCardIndex];
    }

    function requireDuelPlayer(address player) internal view returns (uint8){
        if(player == _duel.player1){
            return 1;
        }else if(player == _duel.player2){
            return 2;
        }
        revert InvalidDeck(player);
    }

    error AlreadyShownCard(uint8 cardIndex);
    
    function showOpponentHandCard(
        uint8 cardIndex,
        uint256[2] calldata revealToken,
        uint256[8] calldata proof
    ) external {
        uint8 playerId = requireDuelPlayer(msg.sender);
        uint8 targetPlayerId = playerId == 1 ? 2 : 1;

        uint256[2][][] storage reveals = targetPlayerId == 1
            ? _duel.player1Reveals
            : _duel.player2Reveals;
        // uint8[] storage playerHand= targetPlayerId ==1 ? _duel.player1Hand: _duel.player2Hand;

        if (reveals[cardIndex].length > 0) {
            revert AlreadyShownCard(cardIndex);
        }

        _requireValidRevealToken(
            targetPlayerId,
            cardIndex,
            playerId,
            revealToken,
            proof
        );

        reveals[cardIndex].push(revealToken);
        
        //Sanity check
        if(targetPlayerId==1){
            require(_duel.player1Reveals[cardIndex].length == 1 
            && _duel.player1Reveals[cardIndex][0].length ==2 , "Player1 reveals not updated");
        }else{
            require(_duel.player2Reveals[cardIndex].length == 1
            && _duel.player2Reveals[cardIndex][0].length ==2 , "Player2 reveals not updated");
        }

        if (_duel.state == DuelState.RevealInitialHand) {
            bool allRevealed = true;
            for (uint8 i = 0; i < INIT_HAND_SIZE; i++) {
                if (reveals[i].length == 0) {
                    allRevealed = false;
                    break;
                }
                
            }

            
            if (allRevealed) {
                if (playerId == 1) {
                    _duel.done1 = true;
                } else {
                    _duel.done2 = true;
                }
            }
        }

        if (_duel.done1 && _duel.done2) {
            _nextDuelState();
        }
    }

    function playCard(
    uint8 cardIndex,
    uint256[2] calldata revealToken,
    uint256[8] calldata proof
    ) external {
        // Some simple checks...
        uint8 playerId = requireDuelPlayer(msg.sender);
        
        uint256[2][][] storage reveals = playerId == 1
            ? _duel.player1Reveals
            : _duel.player2Reveals;

        _requireValidRevealToken(
            playerId,
            cardIndex,
            playerId,
            revealToken,
            proof
        );

        reveals[cardIndex].push(revealToken);

        if (playerId == 1) {
            _duel.player1Board.push(_realCardId(playerId, cardIndex));
        } else {
            _duel.player2Board.push(_realCardId(playerId, cardIndex));
        }
    }

    function getCardIndex(Point memory unmaskPoint) internal pure returns (uint){
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
