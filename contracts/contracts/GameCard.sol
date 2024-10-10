// SPDX-License-Identifier: GPL-3.0

pragma solidity ^0.8.0;
import '@openzeppelin/contracts/token/ERC721/ERC721.sol';

interface IGameCard {
    function ownerOf(uint256 tokenId) external view returns (address);
    function getAttack(uint256 tokenId) external view returns (uint8) ;
}

contract GameCard is ERC721{

    uint256 private _tokenIdCounter;

    constructor(string memory name, string memory symbol) ERC721(name,symbol){}

    mapping(uint256  => uint8) internal _attacks;

    function getAttack(uint256 tokenId) external view returns (uint8) {
        return _attacks[tokenId];
    }

    function batchMint(address to, uint8 amount) external {
        for(uint8 i=0;i<amount;i++){
            _mintWithAttack(to,++_tokenIdCounter);
        }
    }

    function _mintWithAttack(address to, uint256 id) internal {
        _safeMint(to,id);
        _attacks[id]=_createAttackValue(id);
    }

    function _createAttackValue(uint256 tokenId) internal view returns (uint8) {
        uint8 atk = uint8(uint256(keccak256(abi.encodePacked(block.timestamp,tokenId,msg.sender))));
        return atk;
    }
}