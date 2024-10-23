// SPDX-License-Identifier: GPL-3.0

pragma solidity ^0.8.20;
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import { Point, MaskedCard} from '@zypher-game/secret-engine/Verifiers.sol';//ZgShuffleVerifier, ZgRevealVerifier,
// import {ShuffleService} from "./shuffle/ShuffleService.sol";
import {IRaceGameCardA} from './RaceGameCardA.sol';
import {IState} from './State.sol';
import {ConfirmedOwner} from "@chainlink/contracts/src/v0.8/shared/access/ConfirmedOwner.sol";
import {VRFConsumerBase} from "./vendor/binance/VRFConsumerBase.sol";
import {VRFCoordinatorContract} from "./vendor/binance/VRFCoordinator.sol";
import {IVRF} from './GameVRF.sol';
import {IRaceZypher} from './RaceZypher.sol';
 
interface IRaceGame{
    function mintCallback(address to,uint256[] memory _randomWords,uint8 minAllowedRarity) external ;
    function envCardCallback(uint256 matchIndex, uint256[] memory _randomWords) external;
}

contract RaceGame is Ownable{
    uint256 private constant VALID_DECK_SIZE = 20;
    uint256 private constant INIT_HAND_SIZE = 3;
    uint256 private constant TOTAL_HAND_SIZE = 3;
    uint32 private constant MAX_BOARD_SIZE=20;
    uint32 public constant INIT_MINT_COUNT=20;
    uint8 private constant WINNING_SCORE=20;

    //players deck
    event EnvDeckCreated(uint256 size);
    mapping(uint256=>IState.EnvCard[]) public envMatchCards;
    // mapping(address => uint256[]) private _decks;
    //match winners
    mapping(address=>uint256) public winCount;
    //player match mapping
    mapping(address=>uint256) public currentMatch;
    //matches
    mapping(uint256=>IState.Match) public matches;
    uint _matchCounter;

    // ShuffleService public _shuffleService;
    // ZgRevealVerifier public _reveal;
    IRaceZypher _zypher;
    IVRF public _vrf;
    event RequestSent(uint256 requestId, uint32 numWords);
    

   

    IRaceGameCardA cardSet;

    constructor(address _card) 
        Ownable(msg.sender)
    //address vrfAddress
    //  ConfirmedOwner(msg.sender)
    //     VRFConsumerBase(vrfAddress)
    {
        cardSet = IRaceGameCardA(_card);
        _matchCounter=1;
    }

    function updateCardSet(address cardSetAddress) external onlyOwner{
        cardSet = IRaceGameCardA(cardSetAddress );
    }

    function _nextMatchState(IState.Match storage _match) internal returns (IState.MatchState) {
        _match.state = IState.MatchState(uint256(_match.state) + 1);
        for(uint8 i=0;i<_match.playerCount;i++){
            _match.players[i].done=false;
        }
        return _match.state;
    }


    error InvalidState(uint256 expectedState, uint256 actualState);    
    error PlayerAlreadyInit(address player);
    modifier isPlayerInit(address player){
        if (cardSet.getAllCards(player).length<=0)
            revert PlayerAlreadyInit(player);
        _;
    }

     modifier isPlayerNotInit(address player){
        if (cardSet.getAllCards(player).length>0)
            revert PlayerAlreadyInit(player);
        _;
    }

    error MatchAlreadyExistsForPlayer(address player);
    modifier isMatchEmpty(address player){
        uint256 matchId=currentMatch[player];
        if(matchId<_matchCounter){
            IState.Match storage mt = matches[currentMatch[player]];
            if(mt.playerCount >0 && mt.state==IState.MatchState.Finished)
                revert MatchAlreadyExistsForPlayer(player);
        }
        _;
    }

    error MatchDoesNotExists(uint256 matchIndex);
    modifier isMatchExists(uint256 matchIndex){
        if(matchIndex>=_matchCounter || matchIndex == 0){
            revert MatchDoesNotExists(matchIndex);
        }
        _;
    }
 
    //function setVerifiers(ShuffleService shuffle, ZgRevealVerifier reveal) external onlyOwner{
    function setZypher(IRaceZypher zypher) external onlyOwner{
        // _shuffleService=shuffle;
        // _reveal = reveal;
        //_zypher.setVerifiers(shuffle,reveal);
        _zypher=zypher;
    }

    function setVRF(address vrf) external onlyOwner{
        _vrf=IVRF(vrf);
    }

    //----------------INIT PLAYER ---------------------
    error PayableError(uint256 expected, uint256 actual);
    function initPlayer(uint topup) external payable isPlayerNotInit(msg.sender){
        address player = msg.sender;
        //Mint cards
        if(topup==1){
            _vrf.topUpSubscription{value: 10**14}();
        }else {
            if(msg.value>0)
                revert PayableError(0, msg.value);
        }
        batchMint(player, INIT_MINT_COUNT, uint8(IState.CardRarity.A),0);   
    }


    function batchMint(address to, uint32 nftCount, uint8 minAllowedType,uint256 matchId) internal {
        uint256 requestId=_vrf.requestRandomWords(to,nftCount,minAllowedType,matchId,0);
        emit RequestSent(requestId, nftCount);
    }
     

    //----------------CREATE NEW MATCH---------------------
    function createNewMatch(Point memory publicKey, uint8 playerCount,uint topup
    ) external payable
     isPlayerInit(msg.sender)
     isMatchEmpty(msg.sender){
        if(topup==1){
            _vrf.topUpSubscription{value: 10**14}();
        }else {
            if(msg.value>0)
                revert PayableError(0, msg.value);
        }
        address creator=msg.sender;
        //check if player is init
        //check match exists
        uint256 matchId=_createNewMatch(publicKey, creator,playerCount);
        //Todo!:Should not be dependent on match
        _setEnvDeck(matchId);
    }

    function _setEnvDeck(uint256 matchId) internal{
        uint256 requestId=_vrf.requestRandomWords(address(0),uint32(VALID_DECK_SIZE),0,matchId,1);
        emit RequestSent(requestId, uint32(VALID_DECK_SIZE));
    }
    
    function _createNewMatch(Point memory publicKey, address creator,uint8 playerCount) internal returns(uint256){
        uint256 counter = _matchCounter++;
        IState.Match storage _match = matches[counter];
        _match.state=IState.MatchState.None;
        _match.playerCount=playerCount;
        _match.creator=creator;
        
        _match.players[0].player=creator;
        _match.players[0].publicKey=publicKey;
        _match.players[0].playerReveals = new uint256[2][][](VALID_DECK_SIZE);
        _match.players[0].playerIndex=0;
        _match.playerTurn=0;
        _match.envDeck.envReveals = new uint256[2][][](VALID_DECK_SIZE);
        // _match.envDeck.originalCards = new uint256[](VALID_DECK_SIZE);
        currentMatch[creator]=counter;
        return counter;
    }

    error CardOwnerMismatch(address e, address a);
    modifier isPlayerCardsMatch(uint256[] memory cards){
        address player = msg.sender;
        for(uint i=0;i<cards.length;i++){
            if(cardSet.ownerOf(cards[i])!=player){
                revert CardOwnerMismatch(player,cardSet.ownerOf(cards[i]));
            }
        }
        _;
    }
    error NotCreator(address expected, address actual);
    function setCreatorDeck(uint256 matchIndex, uint256[] memory originalCardIndex) external
    isMatchExists(matchIndex) 
    isPlayerCardsMatch(originalCardIndex) {
        address player=msg.sender;
        IState.Match storage _match=matches[matchIndex];
        if(player != _match.players[0].player){
            revert NotCreator(_match.players[0].player,player);
        }
        _match.players[0].originalCards=originalCardIndex;
        _match.players[0].nextRoundPlayerRevealCount=uint8(INIT_HAND_SIZE);
        _match.playerTurn=1;
    }

    //----------------JOIN MATCH---------------------
    error WaitingForFirstPlayerDeck();
    error InvalidPubkey(uint8 playerIndex);
    function joinMatch(uint256 matchIndex,Point memory publicKey,uint256[] memory originalCardIndex) external
    isMatchExists(matchIndex) 
    isPlayerCardsMatch(originalCardIndex) {
        IState.Match storage _match = matches[matchIndex];
        if(_match.playerTurn == 0){
            revert WaitingForFirstPlayerDeck();
        }
        address player = msg.sender;
        //Todo: Check if user has not joined any other match
        _match.players[1].player=player;
        _match.players[1].publicKey=publicKey;
        _match.players[1].originalCards=originalCardIndex;
        _match.players[1].playerReveals = new uint256[2][][](VALID_DECK_SIZE);
        _match.players[1].nextRoundPlayerRevealCount=uint8(INIT_HAND_SIZE);
        _match.players[1].playerIndex=1;
        _match.playerTurn=2;
        currentMatch[player]=matchIndex;
        if(_match.playerTurn == _match.playerCount){
            _match.playerTurn=0;
            Point[] memory playerKeys = new Point[](_match.playerCount);
             playerKeys[0]=_match.players[0].publicKey;
             playerKeys[1]=_match.players[1].publicKey;
             for(uint8 i=0;i<playerKeys.length;i++){
                if(playerKeys[i].x==0){
                    revert InvalidPubkey(i);
                }
             }
            //_match.gameKey = _reveal.aggregateKeys(playerKeys);
            _match.gameKey = _zypher.aggregateKeys(playerKeys);

            //
            _nextMatchState(_match);
        }
    }

    function getMatchEnvCards(uint256 matchIndex) public view 
    isMatchExists(matchIndex)
    returns (IState.EnvCard[] memory)
    {
        return envMatchCards[matchIndex];
    }

    function getPlayerData(uint256 matchIndex, address playerAddress) public view
    isMatchExists(matchIndex)
    returns (IState.PlayerMatchData memory)
    {
        IState.Match storage _match= matches[matchIndex];
        uint8 playerIndex=getPlayerIndex(_match,matchIndex, playerAddress);
        if(currentMatch[playerAddress] != matchIndex) {
            revert PlayerNotPartOfMatch(currentMatch[playerAddress],matchIndex);
        }
        return _match.players[playerIndex];
    } 

    function getPKC(uint256 matchIndex) public view
    isMatchExists(matchIndex)
    returns (uint256[24] memory)
    {
        IState.Match storage _match= matches[matchIndex];
        return _match.pkc;
    } 

    function getPlayerDataByIndex(uint256 matchIndex, uint8 playerIndex) public view
    isMatchExists(matchIndex)
    returns (IState.PlayerMatchData memory)
    {
        IState.Match storage _match= matches[matchIndex];
        return _match.players[playerIndex];
    } 

    function setJointKey(uint256 matchIndex,uint256[24] calldata pkc) external
    isMatchExists(matchIndex){
        IState.Match storage _match= matches[matchIndex];
        _match.pkc = pkc;
        uint256[] memory tempPkc=new uint256[](IState.PKC_SIZE);
        for (uint i=0;i<pkc.length;i++){
            tempPkc[i]=pkc[i];
        }
        // _shuffleService.setPkc(tempPkc);
        _zypher.setPkc(tempPkc);
        _nextMatchState(_match);
    }

    // function _shuffle(
    //     uint256[4][] calldata maskedDeck,
    //     uint256[4][] calldata shuffledDeck,
    //     bytes calldata proof) internal{
    //     uint256[] memory maskedDeckInput = new uint256[](VALID_DECK_SIZE*4);
    //     uint256[] memory shuffledDeckInput = new uint256[](VALID_DECK_SIZE*4);
        
    //     for (uint256 i = 0; i < VALID_DECK_SIZE; i++) {
    //         maskedDeckInput[i * 4 + 0] = maskedDeck[i][0];
    //         maskedDeckInput[i * 4 + 1] = maskedDeck[i][1];
    //         maskedDeckInput[i * 4 + 2] = maskedDeck[i][2];
    //         maskedDeckInput[i * 4 + 3] = maskedDeck[i][3];

    //         shuffledDeckInput[i * 4 + 0] = shuffledDeck[i][0];
    //         shuffledDeckInput[i * 4 + 1] = shuffledDeck[i][1];
    //         shuffledDeckInput[i * 4 + 2] = shuffledDeck[i][2];
    //         shuffledDeckInput[i * 4 + 3] = shuffledDeck[i][3];
    //     }
    //     _shuffleService.setDeck(maskedDeckInput);
    //     _shuffleService.verify(shuffledDeckInput, proof);
    // }

    //Shuffle and submit self deck
    error InvalidDeck(address player);
    error InvalidShuffle();
    error FailedToShuffleEnvDeck(uint256 state);
    error CannotMaskAlreadyMaskedDeck( uint256 shuffleCount);
    function maskEnvDeck(uint256 matchIndex,
        uint256[4][] calldata maskedDeck,
        uint256[4][] calldata shuffledDeck,
        bytes calldata proof
    ) external{
        IState.Match storage _match= matches[matchIndex];
        if(_match.state != IState.MatchState.ShuffleEnvDeck){
            revert FailedToShuffleEnvDeck(uint256(_match.state));
        }
        
        address player = msg.sender;
        uint8 playerIndex=getPlayerIndex(_match,matchIndex, player);
        if(currentMatch[player] != matchIndex) {
            revert PlayerNotPartOfMatch(currentMatch[player],matchIndex);
        }
        
        if(_match.envDeck.shuffleCount>>playerIndex & 1 == 1){
            revert CannotMaskAlreadyMaskedDeck( uint256(_match.envDeck.shuffleCount));
        }
        _match.envDeck.shuffleCount+=1<<playerIndex;

        //_shuffle(maskedDeck,shuffledDeck,proof);
        _zypher.shuffle(maskedDeck, shuffledDeck, proof);
 
        //Store deck
        _match.envDeck.cards=shuffledDeck;
        //_match.envDeck.shuffleCount=1;
        
    }
    

    error CannotShuffleUnmaskedDeck(uint256 shuffleCount);
    function shuffleEnvDeck(
        uint256 matchIndex,
        uint256[4][] calldata currentDeck ,
        uint256[4][] calldata shuffledDeck,
        bytes calldata proof
    ) external 
    isMatchExists(matchIndex) {
        // Do some simple checks...
        IState.Match storage _match= matches[matchIndex];
        address player = msg.sender;
        if(currentMatch[player] != matchIndex) {
            revert PlayerNotPartOfMatch(currentMatch[player],matchIndex);
        }
        if(_match.envDeck.shuffleCount == 0 ){
            revert CannotShuffleUnmaskedDeck(uint256(_match.envDeck.shuffleCount));
        }
        //
        //uint256[4][] storage currentDeck = _match.envDeck.cards;
        uint8 playerIndex=getPlayerIndex(_match,matchIndex, player);

        //_shuffle(currentDeck,shuffledDeck,proof);
        _zypher.shuffle(currentDeck, shuffledDeck, proof);

        //Store deck
        _match.envDeck.cards=shuffledDeck;
        _match.envDeck.shuffleCount+=1<<playerIndex;
        //Goto next state if done
        uint8 max=uint8(2**( _match.playerCount)-1);
        if(_match.envDeck.shuffleCount==max){
            _match.envDeck.shuffleCount=0;
            _nextMatchState(_match);
        }
        

    }
    function submitDeck(
        uint256 matchIndex,
        //uint256[24] calldata pkc,
        uint256[4][] calldata maskedDeck,
        uint256[4][] calldata shuffledDeck,
        bytes calldata proof
    ) external 
    isMatchExists(matchIndex) {
        // Do some simple checks...
        IState.Match storage _match= matches[matchIndex];
        if(_match.state != IState.MatchState.SubmitSelfDeck){
            revert InvalidState(uint256(IState.MatchState.SubmitSelfDeck),uint256(_match.state));
        }

        //_shuffle(maskedDeck,shuffledDeck,proof);
         _zypher.shuffle(maskedDeck, shuffledDeck, proof);
        // Store deck...
        address player = msg.sender;
        bool invalidPlayer = true;
        for(uint8 i=0;i<_match.playerCount;i++){
            if(_match.players[i].player==player){
                invalidPlayer=false;
                _match.players[i].playerDeck=shuffledDeck;
                _match.players[i].done=true;
            }
        } 
        if(invalidPlayer){
            revert InvalidDeck(player);
        }
        bool toNextState=true;
        for(uint8 i=0;i<_match.playerCount;i++){
            if(_match.players[i].done==false){
                toNextState=false;
            }
        }
        if(toNextState)
            _nextMatchState(_match);
    }


    error PlayerNotPartOfMatch(uint256,uint256);
    error ShuffleDeckError(address);
    //Shuffle and submit other players deck
    function shuffleOtherDeck(
        uint256 matchIndex,
        //uint8 playerIndex,
        address playerAddress,
        uint256[4][] calldata currentDeck, //VALID_DECK_SIZE
        uint256[4][] calldata shuffledDeck, //VALID_DECK_SIZE
        bytes calldata proof
    ) external 
    isMatchExists(matchIndex) {
        // Do some simple checks...
        IState.Match storage _match= matches[matchIndex];
        address player = msg.sender;
        if(currentMatch[player] != matchIndex) {
            revert PlayerNotPartOfMatch(currentMatch[player],matchIndex);
        }
        //
        uint8 playerIndex=getPlayerIndex(_match,matchIndex, playerAddress);
        uint8 senderPlayerIndex=getPlayerIndex(_match,matchIndex, player);
        IState.PlayerMatchData storage playerData=_match.players[playerIndex];
        IState.PlayerMatchData storage senderPlayerData=_match.players[senderPlayerIndex];
        //Todo! this only works for 2 players
        if(senderPlayerData.done==true){
            revert ShuffleDeckError(player);
        }
        if(playerData.player == player){
            revert ShuffleDeckError(player);
        }
        if(_match.state != IState.MatchState.ShuffleOpponentDeck){
            revert InvalidState(uint256(IState.MatchState.SubmitSelfDeck),uint256(_match.state));
        }
        //  _shuffle(currentDeck,shuffledDeck,proof);
        _zypher.shuffle(currentDeck, shuffledDeck, proof);
        //Store deck
        playerData.playerDeck=shuffledDeck;
        //Todo! this only works for 2 players
        senderPlayerData.done=true;
        //Goto next state if done
        bool toNextState=true;
        for(uint8 i=0;i<_match.playerCount;i++){
            if(_match.players[i].done==false){
                toNextState=false;
            }
        }
        if(toNextState)
            _nextMatchState(_match);

    }

    // error InvalidRevealToken();
    // function _requireValidRevealToken(
    //     //uint8 targetPlayerId,
    //     uint256[4][] storage targetPlayerDeck, 
    //     uint8 targetCardIndex,
    //     uint8 senderPlayerId,
    //     IState.Match storage _match,
    //     uint256[2] calldata revealToken,
    //     uint256[8] calldata proof
    // ) internal view {
    //     //uint256[4][] storage deck = _match.players[targetPlayerId].playerDeck;
    //     Point storage publicKey = _match.players[senderPlayerId].publicKey;

    //     if (!_reveal.verifyRevealWithSnark([
    //         targetPlayerDeck[targetCardIndex][2],
    //         targetPlayerDeck[targetCardIndex][3],
    //         revealToken[0],
    //         revealToken[1],
    //         publicKey.x,
    //         publicKey.y
    //     ], proof)) {
    //         revert InvalidRevealToken();
    //     }
    // }
    
    function getPlayerIndex(IState.Match storage _match, uint256 matchIndex, address player) internal view returns(uint8) {
        
        bool isSenderAPlayer=false;
        uint8 senderPlayerIndex=0;
        for(uint8 i=0;i<_match.playerCount;i++){
            if(_match.players[i].player == player){
                isSenderAPlayer=true;
                senderPlayerIndex=i;
                break;
            }
        }

        if(isSenderAPlayer==false){
            revert PlayerNotPartOfMatch(currentMatch[player],matchIndex);
        }
        return senderPlayerIndex;
    }

    //----------------Show Deck Card -------------------------
    error AlreadyRevealedByPlayer(uint8 playerIndex);
    function showNextEnvCard(uint256 matchIndex,
        //uint8 cardIndex,
        uint256[2] calldata revealToken,
        uint256[8] calldata proof
    ) external 
    isMatchExists(matchIndex){
        IState.Match storage _match= matches[matchIndex];
        if (_match.rounds==0 && _match.state != IState.MatchState.RevealEnvCard) {
            revert InvalidState(uint256(IState.MatchState.RevealEnvCard),uint256(_match.state));
        }
        if(_match.rounds>0 && _match.playerTurnType != 1){
            revert InvalidState(1,_match.playerTurnType);
        }
        uint8 cardIndex=_match.envDeck.envRevealIndex;
        address player = msg.sender;
        uint8 senderPlayerIndex=getPlayerIndex(_match, matchIndex,player);

        uint256[2][][] storage reveals = _match.envDeck.envReveals;

        if (reveals[cardIndex].length == _match.playerCount) {
            revert AlreadyShownCard(cardIndex);
        }
        
        // _requireValidRevealToken(
        //         _match.envDeck.cards,
        //         cardIndex,
        //         senderPlayerIndex,
        //         _match,
        //         revealToken,
        //         proof
        //     );
        _zypher.requireValidRevealToken(_match.envDeck.cards,
                cardIndex,_match.players[senderPlayerIndex].publicKey,
                revealToken,proof);
        reveals[cardIndex].push(revealToken);
        if(_match.envDeck.shuffleCount>>senderPlayerIndex & 1 == 1){
            revert AlreadyRevealedByPlayer( senderPlayerIndex);
        }
        _match.envDeck.shuffleCount+=1<<senderPlayerIndex; 

        if(reveals[cardIndex].length==_match.playerCount){
            // uint256 maskedCardIndex=_realCardId(_match,_match.envDeck.cards[cardIndex],reveals[cardIndex]);
            uint256 maskedCardIndex=_zypher.realCardId(_match.playerCount,_match.envDeck.cards[cardIndex],reveals[cardIndex]);
            _match.envDeck.envBoard=maskedCardIndex;
            _match.revealEnv=1;
            _match.playerTurnType=2;
            _match.envDeck.envRevealIndex+=1;
            _match.envDeck.shuffleCount=0;
            if (_match.rounds==0){
                _nextMatchState(_match);
                
            }else{
                //
                uint8 currentRound=_match.rounds;
                calculateNextTurn(_match,matchIndex);
                //If round is same then it means next player turn is there
                if (currentRound == _match.rounds){
                    _match.players[_match.playerTurn].nextRoundPlayerRevealCount=_match.players[_match.playerTurn].playerRevealCount+1;
                }
            }
            //Goto next state
            
        }
    }


    //----------------Show Opponent Cards -------------------------
    error AlreadyShownCard(uint8 cardIndex);
    error CardRevealCountError(uint8 expectedCount, uint8 actualCount);
    // error RevealCountMismatch(uint8 expectedCount, uint8 actualCount);
    error CannotShowPlayerCards(uint256 expectedState,uint8 actualState);
    function showOpponentCards(
        uint256 matchIndex,
        uint8 playerIndex,
        //uint8[] calldata cardIndices,
        uint8 cardCount,
        uint256[2][] calldata revealTokens,
        uint256[8][] calldata proofs
    ) external 
    isMatchExists(matchIndex) {
        IState.Match storage _match= matches[matchIndex];
        address player = msg.sender;
        uint8 senderPlayerIndex=getPlayerIndex(_match, matchIndex,player);
        if(_match.rounds>0 && cardCount!=1){
            revert CardRevealCountError(1,cardCount);
        }
        IState.PlayerMatchData storage playerData=_match.players[playerIndex];
        if(playerData.playerRevealProofIndex>>senderPlayerIndex & 1 == 1){
            revert AlreadyRevealedByPlayer(senderPlayerIndex);
        }
        // if(playerData.playerRevealCount+cardCount!=playerData.nextRoundPlayerRevealCount) {
        //     revert CardRevealCountError(uint8(playerData.nextRoundPlayerRevealCount),playerData.playerRevealCount+cardCount);
        // }
        if (_match.rounds==0 && _match.state != IState.MatchState.RevealPlayersHand) {
            revert InvalidState(uint256(IState.MatchState.RevealPlayersHand),uint256(_match.state));
        }
        if(_match.rounds>0 && _match.playerTurnType != 2){
            revert CannotShowPlayerCards(2,_match.playerTurnType);
        }

        uint256[2][][] storage reveals = playerData.playerReveals;
        // uint8 cardStartIndex=;
        for(uint8 i=playerData.playerRevealCount;i<(playerData.playerRevealCount+cardCount);i++){
            //Todo! This check only works for 2players
            if (reveals[i].length > 0) {
                revert AlreadyShownCard(i);
            }
        }

        for(uint8 i=0;i<(cardCount);i++){
            // _requireValidRevealToken(
            //     playerData.playerDeck,
            //     i+playerData.playerRevealCount,
            //     senderPlayerIndex,
            //     _match,
            //     revealTokens[i],
            //     proofs[i]
            // );
            _zypher.requireValidRevealToken(playerData.playerDeck,
                 i+playerData.playerRevealCount,_match.players[senderPlayerIndex].publicKey,
                revealTokens[i],proofs[i]);
            reveals[i+playerData.playerRevealCount].push(revealTokens[i]);
            //reveals[i][senderPlayerIndex]=revealTokens[i-cardStartIndex];
        }

        bool allCardsRevealed = true;
        for(uint8 i=0;i<(cardCount);i++){
            if (reveals[i+playerData.playerRevealCount].length == 0) {
                allCardsRevealed = false;
                break;
            }
            
        }
        if (allCardsRevealed) {
            playerData.playerRevealCount+=cardCount;
            playerData.playerRevealProofIndex+=1<<senderPlayerIndex;
        }
        bool addToHand=true;
        for(uint8 j=0;j<_match.playerCount;j++){
            if(playerIndex!=j){
                if(playerData.playerRevealProofIndex>>j & 1 == 0 ){
                    addToHand=false;
                }
            }
        }
        if (addToHand==true) {
            for(uint8 i=playerData.playerRevealCount-cardCount;i<(playerData.playerRevealCount);i++){
                playerData.playerHand.push(i);
               
            }
        }

        //To next state
        
        if (_match.rounds==0){
            bool toNextState=true;
            for(uint8 i=0;i<_match.playerCount;i++){
                if (_match.players[i].playerHand.length < 1){
                    toNextState=false;
                }
            }
            if(toNextState==true){
                _nextMatchState(_match);
            }
        }
    }

    // function _realCardId(
    //     IState.Match storage _match,
    //     uint256[4] storage maskedCard,
    //     uint256[2][] storage reveals
    //     //uint256[] storage originalCards
    // ) internal view returns (uint256) {
 
    //     if (reveals.length < _match.playerCount) {
    //         return 0;
    //     }

    //     Point[] memory rTokens = new Point[](reveals.length);
    //     for (uint256 i = 0; i < reveals.length; i++) {
    //         rTokens[i] = Point(reveals[i][0], reveals[i][1]);
    //     }

    //     // Point memory realCardPoint = _reveal.unmask(MaskedCard(
    //     //     maskedCard[0],
    //     //     maskedCard[1],
    //     //     maskedCard[2],
    //     //     maskedCard[3]
    //     // ), rTokens);
    //     Point memory realCardPoint = _zypher.unmask(maskedCard, rTokens);

    //     uint realCardIndex=IState.getCardIndex(realCardPoint);
    //     return realCardIndex;
    //     //return originalCards[realCardIndex];
    // }

    //----------------Play Card -------------------------

    error PlayerTurnError(uint8 expectedTurn, uint8 actualTurn);
    error TurnAlreadyStarted();
    error EnvCardCanBeUpdatedOncePerTurn();
    function playerAction(uint256 matchIndex,uint8 showEnvCard) external 
    isMatchExists(matchIndex){
        IState.Match storage _match= matches[matchIndex];
        address player = msg.sender;
        uint8 senderPlayerIndex=getPlayerIndex(_match, matchIndex,player);
        
        if(_match.playerTurn != senderPlayerIndex){
            revert PlayerTurnError(senderPlayerIndex,_match.playerTurn);
        }

        if(_match.turnStart==1){
            revert TurnAlreadyStarted();
        }

        if(_match.revealEnv==1 && showEnvCard ==1 ){
            revert EnvCardCanBeUpdatedOncePerTurn();
        }

        _match.turnStart=1;
        //If 1 then env card will be revealed
        _match.revealEnv=showEnvCard;
        if(showEnvCard==0){
            _match.playerTurnType=2;
            for(uint8 i=0;i<_match.playerCount;i++){
                _match.players[i].nextRoundPlayerRevealCount=_match.players[i].playerRevealCount+1;
            }
        }
        else
            _match.playerTurnType=1;
    }


    error CannotPlayCardWhenEnvIsSelected();
    function playCardOnDeck(uint256 matchIndex,uint8 playerHandIndex,
    uint256[2] calldata revealToken,
    uint256[8] calldata proof) external
     isMatchExists(matchIndex) {
        IState.Match storage _match= matches[matchIndex];
        address player = msg.sender;
        uint8 senderPlayerIndex=getPlayerIndex(_match, matchIndex,player);
        uint256[2][][] storage reveals = _match.players[senderPlayerIndex].playerReveals;
        
        if(_match.playerTurn!=senderPlayerIndex){
            revert PlayerTurnError( senderPlayerIndex,_match.playerTurn);
        }

        if(_match.playerTurnType != 2){
            revert CannotShowPlayerCards(2,_match.playerTurnType);
        }
        IState.PlayerMatchData storage playerData=_match.players[senderPlayerIndex];
        uint8 cardIndex=playerData.playerHand[playerHandIndex];
        // _requireValidRevealToken(
        //         _match.players[senderPlayerIndex].playerDeck,
        //         cardIndex,
        //         senderPlayerIndex,
        //         _match,
        //         revealToken,
        //         proof
        //     );
        _zypher.requireValidRevealToken( _match.players[senderPlayerIndex].playerDeck,
                cardIndex,_match.players[senderPlayerIndex].publicKey,
                revealToken,proof);
        reveals[cardIndex].push(revealToken);
        //Reveal Index will be player 
        // reveals[cardIndex][senderPlayerIndex][0]=revealToken[0];
        // reveals[cardIndex][senderPlayerIndex][1]=revealToken[1];
        //Show card of player board
        //uint256 maskedCardIndex=_realCardId(_match,playerData.playerDeck[cardIndex],reveals[cardIndex]);
        uint256 maskedCardIndex=_zypher.realCardId(_match.playerCount,playerData.playerDeck[cardIndex],reveals[cardIndex]);
        playerData.playerBoard=playerData.originalCards[maskedCardIndex];
        //Update player hand
        playerData.playerHand[playerHandIndex]=playerData.playerHand[playerData.playerHand.length-1];
        playerData.playerHand.pop();
        
        playerData.playerRevealProofIndex+=1<<senderPlayerIndex;
        // _match.players[]
        calculateNextTurn(_match,matchIndex);
    }

    function calculateNextTurn(IState.Match storage _match,uint256 matchIndex) internal{
        //Turn reset
        _match.turnStart=0;
        
        //if(_match.orderReverse==1){
        if(_match.rounds%2==1){
            if(_match.playerTurn<=0){
                //Next round
                scoreRound(_match,matchIndex);
            }else{
                _match.playerTurn=_match.playerTurn-1;
            }
        }else{
             _match.playerTurn=_match.playerTurn+1;
            if(_match.playerTurn>=_match.playerCount){
                //Next round
                scoreRound(_match,matchIndex);
            }
        }
    }
 
 
    
    //----------------Score round -------------------------
    function scoreRound(IState.Match storage _match,uint256 matchIndex) internal {
        IState.EnvCard[] storage _envMatchCards=envMatchCards[matchIndex];
        IState.EnvCard storage envCard= _envMatchCards[_match.envDeck.envBoard];
        IState.AnimalCard[] memory playerCards=new IState.AnimalCard[](_match.playerCount);
        for(uint8 i=0;i<_match.playerCount;i++){
            playerCards[i] =cardSet.getCardProp( _match.players[i].playerBoard);
        }
        uint256[] memory scores=IState.calculateRoundScore(playerCards,envCard);
        _match.rounds+=1;
        for(uint8 i=0;i<scores.length;i++){
            _match.players[i].position += scores[i];
            _match.players[i].playerRevealProofIndex=0;
            if(_match.players[i].position>=WINNING_SCORE){
                //win
                // _match.isFinished=1;
                winCount[_match.players[i].player]+=1 ;
                _match.state= IState.MatchState.Finished;
                _match.winner=_match.players[i].player;
                for(uint8 j=0;j<_match.playerCount;j++){
                    currentMatch[_match.players[j].player]=0; 
                }
                if(winCount[_match.players[i].player]%3==0){
                    batchMint(_match.players[i].player, 1, uint8(IState.CardRarity.S),matchIndex);
                }else{
                    batchMint(_match.players[i].player, 1, uint8(IState.CardRarity.A),matchIndex);
                }
                return;
            }
        }
        //Reset states
        //_match.orderReverse=1-_match.orderReverse;
        _match.revealEnv=0;
        _match.playerTurnType =0;
        if(_match.rounds%2==1)
            _match.playerTurn=_match.playerCount-1;
        else
            _match.playerTurn=0;
        _match.state=IState.MatchState.PlayerPlayCard;
    }



    ///-------------------VRF---------------------------
    modifier isVRFContract(){
        require (msg.sender == address(_vrf), "Not VRF contract");
        _;
    }

    modifier isVRFCoordinatorContract(){
        require (msg.sender ==  _vrf.getCoordinatorAddress(), "Not VRF Coordinator contract");
        _;
    }
 
    function mintCallback(address to,uint256[] memory _randomWords,uint8 minAllowedRarity) external
    isVRFContract 
    {
        
        cardSet.mintRandomNft(to,
            _randomWords,minAllowedRarity,_randomWords.length/2);
    }

    function envCardCallback(uint256 matchIndex,uint256[] memory _randomWords) external 
    isMatchExists(matchIndex)
    isVRFContract{
        // Match storage _match =matches[matchIndex];
        for(uint8 i=0;i<_randomWords.length;i+=2){
            //_match.envDeck.originalCards[i/2]=IState.getEnvCard( _randomWords[i],_randomWords[i+1]);
            envMatchCards[matchIndex].push((IState.getEnvCard( _randomWords[i],_randomWords[i+1])));
        }
        
        emit EnvDeckCreated(envMatchCards[matchIndex].length);
    }
    
}