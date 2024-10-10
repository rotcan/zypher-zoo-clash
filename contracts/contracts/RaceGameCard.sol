// SPDX-License-Identifier: GPL-3.0

pragma solidity ^0.8.20;
import '@openzeppelin/contracts/token/ERC721/ERC721.sol';

import {IState} from './State.sol';

interface IRaceGameCard{
    function getAllCards(address user) external view returns(uint256[] memory cards);
    function getCardProp(uint256 index) external view returns(IState.AnimalCard memory card);
    function mintRandomNft( address to,uint256 randomWord, uint256 randomWord2, uint8 minRarity) external;
    function ownerOf(uint256 tokenId) external view returns (address);
}
 
contract RaceGameCard is ERC721{
   
    address private _gameContract;

    //user Cards
    mapping(address=>uint256[] ) public userCards;
    //all cards
    mapping(uint256 =>IState.AnimalCard) public allCards;

    uint256 private _tokenIdCounter;


    constructor(string memory name, string memory symbol) ERC721(name,symbol){
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

    function mintRandomNft( address to,uint256 randomWord, uint256 randomWord2, uint8 minRarity) 
    external 
    // isGameContract() 
    {
        _safeMint(to,++_tokenIdCounter);
        IState.AnimalCard memory card = IState.createCard(randomWord,randomWord2,minRarity);
        userCards[to].push(_tokenIdCounter);
        allCards[_tokenIdCounter]=card;
    }


     
    event Received(address, uint256);

    receive() external payable {
        emit Received(msg.sender, msg.value);
    }
}