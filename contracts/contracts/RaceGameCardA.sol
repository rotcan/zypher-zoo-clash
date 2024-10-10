// SPDX-License-Identifier: GPL-3.0

pragma solidity ^0.8.20;

import 'erc721a-upgradeable/contracts/ERC721AUpgradeable.sol';
import '@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol';

import {IState} from './State.sol';

interface IRaceGameCardA{
    function getAllCards(address user) external view returns(uint256[] memory cards);
    function getCardProp(uint256 index) external view returns(IState.AnimalCard memory card);
    function mintRandomNft( address to,uint256[] memory randomWords, uint8 minRarity,uint256 quantity) external;
    function ownerOf(uint256 tokenId) external view returns (address);
}
 
contract RaceGameCardA is ERC721AUpgradeable, OwnableUpgradeable{
   
    address private _gameContract;

    //user Cards
    mapping(address=>uint256[] ) public userCards;
    //all cards
    mapping(uint256 =>IState.AnimalCard) public allCards;

    // uint256 private _tokenIdCounter;


    // constructor(string memory name, string memory symbol) ERC721(name,symbol){
    // }
    function initialize(string memory name, string memory symbol) initializerERC721A initializer public {
        __ERC721A_init(name, symbol);
        __Ownable_init(msg.sender);
    }
    
    function setGameContract(address contractAddress) external {
        //require (_gameContract == address(0), "Address already set");
        _gameContract=contractAddress;
    }


    modifier isGameContract(){
        require (msg.sender == _gameContract, "Not game contract");
        _;
    }
 
    function getAllCards(address user) external view returns(uint256[] memory cards){
        return userCards[user];
    }

    function getCardProp(uint256 index) external view returns(IState.AnimalCard memory card){
        return allCards[index];
    }

    function getPlayerCardProps(address user) external view returns(uint256[] memory cards, 
    IState.AnimalCard[] memory animalCards){
        cards=userCards[user];
        animalCards=new IState.AnimalCard[](cards.length);
        for(uint i=0;i<cards.length;i++){
            animalCards[i]=allCards[cards[i]];
        }
        return (cards,animalCards);
    }

    function mintRandomNft( address to,uint256[] memory randomWords,  uint8 minRarity,uint256 quantity) 
    external 
    // isGameContract() 
    {
        //_safeMint(to,++_tokenIdCounter);
        uint256 tid=_nextTokenId();
        _mint(to, quantity);
        for(uint256 i=0;i<quantity;i++){
            IState.AnimalCard memory card = IState.createCard(randomWords[i*2],randomWords[i*2+1],minRarity);
            userCards[to].push(i+tid);
            allCards[i+tid]=card;
        }
    }


     
    event Received(address, uint256);

    receive() external payable {
        emit Received(msg.sender, msg.value);
    }
}