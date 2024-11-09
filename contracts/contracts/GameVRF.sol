// SPDX-License-Identifier: GPL-3.0

pragma solidity ^0.8.20;

import {ConfirmedOwner} from "@chainlink/contracts/src/v0.8/shared/access/ConfirmedOwner.sol";
import {VRFConsumerBase} from "./vendor/binance/VRFConsumerBase.sol";
import {VRFCoordinatorContract} from "./vendor/binance/VRFCoordinator.sol";
import {IState} from './State.sol';
import {IRaceGame} from './RaceGame.sol';

interface IVRF{
    function requestRandomWords(
        address to,
        uint32 nftCount,
        uint8 minRarity ,
        uint256 matchId,
        uint8 requestType       
    ) external returns (uint256);
    function getCoordinatorAddress() external view returns(address);
    function topUpSubscription() external payable;
}

contract GameVRF is VRFConsumerBase, ConfirmedOwner{
    //opBNB Testnet
    // address private vrfCoordinatorAddress=0x2B30C31a17Fe8b5dd397EF66FaFa503760D4eaF0;
    //VRFCoordinatorInterface private VRFCOORDINATOR; //


     //opBNB Testnet
    // address private vrfCoordinatorAddress=0x2B30C31a17Fe8b5dd397EF66FaFa503760D4eaF0;
    //VRFCoordinatorInterface private VRFCOORDINATOR; //
    // VRFCoordinatorContract private VRFCOORDINATORCONTRACT;
    // bytes32 private VRF_KEYHASH=0x617abc3f53ae11766071d04ada1c7b0fbd49833b9542e9e91da4d3191c70cc80;

    // event RequestSent(uint256 requestId, uint32 numWords);
    // event RequestFulfilled(
    //     uint256 requestId,
    //     uint256[] randomWords
        
    // );
    // // uint256 payment
    
    // enum RequestType{
    //     Player,
    //     Env
    // }

    // struct RequestStatus {
    //     // uint256 paid; // amount paid in link
    //     bool fulfilled; // whether the request has been successfully fulfilled
    //     uint256[] randomWords;
    //     uint minAllowedRarity;
    //     address to;
    //     uint256 nftCount;
    //     RequestType requestType;
    //     uint256 matchId;
    // }
    // mapping(uint256 => RequestStatus) public s_requests; /* requestId --> requestStatus */

    // past requests Id.
    //uint256[] public requestIds;
    //mapping(uint256 => IState.VRFRequest) 
    // uint256 public lastRequestId;

    // Depends on the number of requested values that you want sent to the
    // fulfillRandomWords() function. Test and adjust
    // this limit based on the network that you select, the size of the request,
    // and the processing of the callback request in the fulfillRandomWords()
    // function.
    // uint32 public callbackGasLimit = 10000000;
    // uint64 public subscriptionId;

    // The default is 3, but you can set this higher.
    // uint16 public requestConfirmations = 3;

    // For this example, retrieve 2 random values in one request.
    // Cannot exceed VRFV2Wrapper.getConfig().maxNumWords.
    // uint32 public numWords = 20;

    // Address LINK - hardcoded for Sepolia
    // address public linkAddress = 0x779877A7B0D9E8603169DdbD7836e478b4624789;

    // address WRAPPER - hardcoded for Sepolia
    // address public wrapperAddress = 0x195f15F2d49d693cE265b4fB0fdDbE15b1850Cc1;
    
    VRFCoordinatorContract private VRFCOORDINATORCONTRACT;
    bytes32 private VRF_KEYHASH=0x617abc3f53ae11766071d04ada1c7b0fbd49833b9542e9e91da4d3191c70cc80;

    struct RequestStatus {
        // uint256 paid; // amount paid in link
        bool fulfilled; // whether the request has been successfully fulfilled
        uint256[] randomWords;
        uint8 minAllowedRarity;
        address to;
        uint256 nftCount;
        uint8 requestType;
        uint256 matchId;
    }
    mapping(uint256 => RequestStatus) public s_requests; /* requestId --> requestStatus */

    // past requests Id.
    //uint256[] public requestIds;
    //mapping(uint256 => IState.VRFRequest) 
    uint256 public lastRequestId;

    // Depends on the number of requested values that you want sent to the
    // fulfillRandomWords() function. Test and adjust
    // this limit based on the network that you select, the size of the request,
    // and the processing of the callback request in the fulfillRandomWords()
    // function.
    uint32 public callbackGasLimit = 2500000;
    uint64 public subscriptionId;

    // The default is 3, but you can set this higher.
    uint16 public requestConfirmations = 3;

    IRaceGame public gameContract;
    uint8 public disableGame;
    
    event RequestFulfilled(
        uint256 requestId,
        uint256[] randomWords
        
    );

     constructor( address vrfAddress)
     ConfirmedOwner(msg.sender)
        VRFConsumerBase(vrfAddress){
        VRFCOORDINATORCONTRACT=VRFCoordinatorContract(vrfAddress);
        createNewSubscription();
    }

    function setGameContractAddress(address gameAddress) external onlyOwner{
        gameContract=IRaceGame(gameAddress);
    }

    function createNewSubscription() private  {
        subscriptionId=VRFCOORDINATORCONTRACT.createSubscription();
        VRFCOORDINATORCONTRACT.addConsumer(subscriptionId,address(this));
    }

    function topUpSubscription() external payable  {
        require(subscriptionId>0,"Subscription not set");
        VRFCOORDINATORCONTRACT.deposit{value: msg.value}(subscriptionId);
    }

    function setDisableGame(uint8 val)external {
        disableGame=val;
    }

    // function setSubscription(uint64 subId) external onlyOwner {
    //     subscriptionId=subId;
    //     VRFCOORDINATORCONTRACT.addConsumer(subscriptionId,address(this));
    // }

    function getCoordinatorAddress() external view returns(address){
        return address(VRFCOORDINATORCONTRACT);
    }

    function setCallbackGasLimit(uint32 limit) external {
        callbackGasLimit=limit;
    }

    error AddressMismatch(address expected, address actual);
    modifier isGameContract(){
        if (msg.sender != address(gameContract)){
            revert AddressMismatch(address(gameContract),msg.sender);
        }
        _;
    }
 
    function requestRandomWords(
        address to,
        uint32 nftCount,
        uint8 minRarity ,
        uint256 matchId,
        uint8 requestType       
    ) 
    external 
    //isGameContract 
    returns (uint256) {
        
        uint256 requestId;
        //uint256 reqPrice;
        if(requestType<2){
            requestId = VRFCOORDINATORCONTRACT.requestRandomWords(
                    VRF_KEYHASH,
                    subscriptionId,
                    requestConfirmations,
                    callbackGasLimit,
                    nftCount*2
            );

            s_requests[requestId] = RequestStatus({
                //paid: reqPrice,
                randomWords: new uint256[](nftCount*2),
                fulfilled: false,
                minAllowedRarity : minRarity,
                to: to,
                nftCount: nftCount,
                requestType: requestType,
                matchId:matchId
            });
        }
        //requestIds.push(requestId);
        lastRequestId = requestId;
        return requestId;
    }

    function fulfillRandomWords(
        uint256 _requestId,
        uint256[] memory _randomWords
    ) internal override {
        // require(s_requests[_requestId].paid > 0, "request not found");
        s_requests[_requestId].fulfilled = true;
        s_requests[_requestId].randomWords = _randomWords;
        if(s_requests[_requestId].requestType == 0){
            //Mint to user
            if(disableGame==0)
            gameContract.mintCallback(s_requests[_requestId].matchId, s_requests[_requestId].to, _randomWords, s_requests[_requestId].minAllowedRarity);
        }else{
             

            // emit EnvDeckCreated(_envDeck.length);
            if(disableGame==0)
            gameContract.envCardCallback(s_requests[_requestId].matchId, _randomWords);
        }
        emit RequestFulfilled(
            _requestId,
            _randomWords
            // s_requests[_requestId].paid
        );
    }


    function getRequestStatus(
        uint256 _requestId
    )
        external
        view
        returns (  bool fulfilled, uint256[] memory randomWords)
    {
        //require(s_requests[_requestId].paid > 0, "request not found");
        RequestStatus memory request = s_requests[_requestId];
        return (  request.fulfilled, request.randomWords);
    }

}