// SPDX-License-Identifier: MIT
pragma solidity ^0.8.6;

import "./BlockhashStoreInterface.sol";
import "./VRFCoordinatorInterface.sol";
import "./TypeAndVersionInterface.sol";
import "./VRF.sol";
import "./ConfirmedOwner.sol";
import "./VRFConsumerBase.sol";

contract VRFCoordinatorContract is
VRF,
ConfirmedOwner,
TypeAndVersionInterface,
VRFCoordinatorInterface
{
    BlockhashStoreInterface public immutable BLOCKHASH_STORE;

    // We need to maintain a list of consuming addresses.
    // This bound ensures we are able to loop over them as needed.
    // Should a user require more consumers, they can use multiple subscriptions.
    uint16 public constant MAX_CONSUMERS = 100;
    error TooManyConsumers();
    error InsufficientBalance(uint96 balance, uint96 payment);
    error InvalidConsumer(uint64 subId, address consumer);
    error InvalidSubscription();
    error InvalidCalldata();
    error MustBeSubOwner(address owner);
    error PendingRequestExists();
    error MustBeRequestedOwner(address proposedOwner);
    error BalanceInvariantViolated(uint256 internalBalance, uint256 externalBalance); // Should never happen
    error ExceedingTheMaximumRechargeLimit(uint96 subAccountMaxDeposit, uint256 oldSubBalance, uint96 depositAmount);
    error InvalidAddress();
    event FundsRecovered(address to, uint256 amount);
    // We use the subscription struct (1 word)
    // at fulfillment time.
    struct Subscription {
        uint96 balance;
        uint64 reqCount; // For fee tiers
    }
    // We use the config for the mgmt APIs
    struct SubscriptionConfig {
        address owner; // Owner can fund/withdraw/cancel the sub.
        address requestedOwner; // For safely transferring sub ownership.
        // Maintains the list of keys in s_consumers.
        // We do this for 2 reasons:
        // 1. To be able to clean up all keys from s_consumers when canceling a subscription.
        // 2. To be able to return the list of all consumers in getSubscription.
        // Note that we need the s_consumers map to be able to directly check if a
        // consumer is valid without reading all the consumers from storage.
        address[] consumers;
    }
    // Note a nonce of 0 indicates an the consumer is not assigned to that subscription.
    mapping(address => mapping(uint64 => uint64)) /* consumer */ /* subId */ /* nonce */
    private s_consumers;
    mapping(uint64 => SubscriptionConfig) /* subId */ /* subscriptionConfig */
    private s_subscriptionConfigs;
    mapping(uint64 => Subscription) /* subId */ /* subscription */
    private s_subscriptions;

    mapping(uint64 => uint32) /* subId */ /* feeTier */
    private s_designated_subId_fee;

    // We make the sub count public so that its possible to
    // get all the current subscriptions via getSubscription.
    uint64 private s_currentSubId;
    uint96 private s_totalBalance;
    event SubscriptionCreated(uint64 indexed subId, address owner);
    event SubscriptionFunded(uint64 indexed subId, uint256 oldBalance, uint256 newBalance);
    event SubscriptionConsumerAdded(uint64 indexed subId, address consumer);
    event SubscriptionConsumerRemoved(uint64 indexed subId, address consumer);
    event SubscriptionCanceled(uint64 indexed subId, address to, uint256 amount);
    event SubscriptionOwnerTransferRequested(uint64 indexed subId, address from, address to);
    event SubscriptionOwnerTransferred(uint64 indexed subId, address from, address to);

    // Set this maximum to 200 to give us a 56 block window to fulfill
    // the request before requiring the block hash feeder.
    uint16 public constant MAX_REQUEST_CONFIRMATIONS = 200;
    // 5k is plenty for an EXTCODESIZE call (2600) + warm CALL (100)
    // and some arithmetic operations.
    uint256 private constant GAS_FOR_CALL_EXACT_CHECK = 5_000;
    error InvalidRequestConfirmations(uint16 have, uint16 min, uint16 max);
    error GasLimitTooBig(uint32 have, uint32 want);
    error NumWordsTooBig(uint32 have, uint32 want);
    error ProvingKeyAlreadyRegistered(bytes32 keyHash);
    error NoSuchProvingKey(bytes32 keyHash);
    error InsufficientGasForConsumer(uint256 have, uint256 want);
    error NoCorrespondingRequest();
    error IncorrectCommitment();
    error BlockhashNotInStore(uint256 blockNum);
    error PaymentTooLarge();
    error Reentrant();
    struct RequestCommitment {
        uint64 blockNum;
        uint64 subId;
        uint32 callbackGasLimit;
        uint32 numWords;
        address sender;
    }
    bytes32[] private s_provingKeyHashes;
    uint96 private s_withdrawableTokens;
    address private s_withdrawableTokensAddress;
    mapping(uint256 => bytes32) /* requestID */ /* commitment */
    private s_requestCommitments;
    event ProvingKeyRegistered(bytes32 keyHash);
    event ProvingKeyDeregistered(bytes32 keyHash);
    event s_withdrawableTokensAddressChange(address newAddress, address oldAddress);
    event RandomWordsRequested(
        bytes32 indexed keyHash,
        uint256 requestId,
        uint256 preSeed,
        uint64 indexed subId,
        uint16 minimumRequestConfirmations,
        uint32 callbackGasLimit,
        uint32 numWords,
        address indexed sender
    );
    event RandomWordsFulfilled(uint256 indexed requestId, uint256 outputSeed, uint96 payment, uint256 fee, bool success);
    struct Config {
        uint16 minimumRequestConfirmations;
        uint16 maxNumWords;
        uint32 maxGasLimit;
        // Reentrancy protection.
        bool reentrancyLock;
        // Gas to cover oracle payment after we calculate the payment.
        // We make it configurable in case those operations are repriced.
        uint32 gasAfterPaymentCalculation;
        uint96 subAccountMaxDeposit;
    }
    Config private s_config;
    FeeConfig private s_feeConfig;
    struct FeeConfig {
        // Flat fee charged per fulfillment in millionths of bnb
        // So fee range is [0, 2^32/10^6].
        uint32 fulfillmentFlatFeeBNBPPMTier1;
        uint32 fulfillmentFlatFeeBNBPPMTier2;
        uint32 fulfillmentFlatFeeBNBPPMTier3;
        uint32 fulfillmentFlatFeeBNBPPMTier4;
        uint32 fulfillmentFlatFeeBNBPPMTier5;
        uint24 reqsForTier2;
        uint24 reqsForTier3;
        uint24 reqsForTier4;
        uint24 reqsForTier5;
    }
    event ConfigSet(
        uint16 minimumRequestConfirmations,
        uint16 maxNumWords,
        uint32 maxGasLimit,
        uint32 gasAfterPaymentCalculation,
        uint96 subAccountMaxDeposit,
        FeeConfig feeConfig
    );
    bool private nodesWhiteListSwitchStatus;
    mapping(address => bool) private nodesWhiteListConfig;
    event NodeAccessDisabled(address node);
    event NodeAccessAdd(address node);
    error NoAuthFulfillRandomWords(address node);

    constructor(
        address blockhashStore
    ) ConfirmedOwner(msg.sender) {
        BLOCKHASH_STORE = BlockhashStoreInterface(blockhashStore);
    }

    /**
     * @notice Registers a proving key to an oracle.
     * @param publicProvingKey key that oracle can use to submit vrf fulfillments
     */
    function registerProvingKey(uint256[2] calldata publicProvingKey) external onlyOwner {
        bytes32 kh = hashOfKey(publicProvingKey);
        for (uint256 i = 0; i < s_provingKeyHashes.length; i++) {
            if (s_provingKeyHashes[i] == kh) {
                revert ProvingKeyAlreadyRegistered(kh);
            }
        }
        s_provingKeyHashes.push(kh);
        emit ProvingKeyRegistered(kh);
    }

    /**
    * @notice Deregisters a proving key to an oracle.
    * @param publicProvingKey key that oracle can use to submit vrf fulfillments
    */
    function deregisterProvingKey(uint256[2] calldata publicProvingKey) external onlyOwner {
        bytes32 kh = hashOfKey(publicProvingKey);
        for (uint256 i = 0; i < s_provingKeyHashes.length; i++) {
            if (s_provingKeyHashes[i] == kh) {
                bytes32 last = s_provingKeyHashes[s_provingKeyHashes.length - 1];
                // Copy last element and overwrite kh to be deleted with it
                s_provingKeyHashes[i] = last;
                s_provingKeyHashes.pop();
            }
        }
          emit ProvingKeyDeregistered(kh);
    }

    /**
    * @notice Returns the proving key hash key associated with this public key
    * @param publicKey the key to return the hash of
    */
    function hashOfKey(uint256[2] memory publicKey) public pure returns (bytes32) {
        return keccak256(abi.encode(publicKey));
    }

    /**
    * @notice Sets the configuration of the vrfv2 coordinator
    * @param minimumRequestConfirmations global min for request confirmations
    * @param maxGasLimit global max for request gas limit
    * @param gasAfterPaymentCalculation gas used in doing accounting after completing the gas measurement
    * @param feeConfig fee tier configuration
    */
    function setConfig(
        uint16 minimumRequestConfirmations,
        uint16 maxNumWords,
        uint32 maxGasLimit,
        uint32 gasAfterPaymentCalculation,
        uint96 subAccountMaxDeposit,
        FeeConfig memory feeConfig
    ) external onlyOwner {
        if (minimumRequestConfirmations > MAX_REQUEST_CONFIRMATIONS) {
            revert InvalidRequestConfirmations(
                minimumRequestConfirmations,
                minimumRequestConfirmations,
                MAX_REQUEST_CONFIRMATIONS
            );
        }
        s_config = Config({
        minimumRequestConfirmations: minimumRequestConfirmations,
        maxNumWords: maxNumWords,
        maxGasLimit: maxGasLimit,
        gasAfterPaymentCalculation: gasAfterPaymentCalculation,
        reentrancyLock: false,
        subAccountMaxDeposit: subAccountMaxDeposit
        });
        s_feeConfig = feeConfig;
        emit ConfigSet(
            minimumRequestConfirmations,
            maxNumWords,
            maxGasLimit,
            gasAfterPaymentCalculation,
            subAccountMaxDeposit,
            s_feeConfig
        );
    }

    function getConfig()
    external
    view
    returns (
        uint16 minimumRequestConfirmations,
        uint16 maxNumWords,
        uint32 maxGasLimit,
        uint32 gasAfterPaymentCalculation,
        uint96 subAccountMaxDeposit
    )
    {
        return (
        s_config.minimumRequestConfirmations,
        s_config.maxNumWords,
        s_config.maxGasLimit,
        s_config.gasAfterPaymentCalculation,
        s_config.subAccountMaxDeposit
        );
    }

    function getFeeConfig()
    external
    view
    returns (
        uint32 fulfillmentFlatFeeBNBPPMTier1,
        uint32 fulfillmentFlatFeeBNBPPMTier2,
        uint32 fulfillmentFlatFeeBNBPPMTier3,
        uint32 fulfillmentFlatFeeBNBPPMTier4,
        uint32 fulfillmentFlatFeeBNBPPMTier5,
        uint24 reqsForTier2,
        uint24 reqsForTier3,
        uint24 reqsForTier4,
        uint24 reqsForTier5
    )
    {
        return (
        s_feeConfig.fulfillmentFlatFeeBNBPPMTier1,
        s_feeConfig.fulfillmentFlatFeeBNBPPMTier2,
        s_feeConfig.fulfillmentFlatFeeBNBPPMTier3,
        s_feeConfig.fulfillmentFlatFeeBNBPPMTier4,
        s_feeConfig.fulfillmentFlatFeeBNBPPMTier5,
        s_feeConfig.reqsForTier2,
        s_feeConfig.reqsForTier3,
        s_feeConfig.reqsForTier4,
        s_feeConfig.reqsForTier5
        );
    }

    function getTotalBalance() external view returns (uint256) {
        return s_totalBalance;
    }

    function getWithdrawableTokens() external view returns (uint96) {
        return s_withdrawableTokens;
    }

    /**
    * @notice Owner cancel subscription, sends remaining coin directly to the subscription owner.
    * @param subId subscription id
    * @dev notably can be called even if there are pending requests, outstanding ones may fail onchain
    */
    function ownerCancelSubscription(uint64 subId) external onlyOwner {
        if (s_subscriptionConfigs[subId].owner == address(0)) {
            revert InvalidSubscription();
        }
        cancelSubscriptionHelper(subId, s_subscriptionConfigs[subId].owner);
    }

    /**
    * @inheritdoc VRFCoordinatorInterface
    */
    function getRequestConfig()
    external
    view
    override
    returns (
        uint16,
        uint32,
        bytes32[] memory
    )
    {
        return (s_config.minimumRequestConfirmations, s_config.maxGasLimit, s_provingKeyHashes);
    }

    /**
     * @inheritdoc VRFCoordinatorInterface
   */
    function requestRandomWords(
        bytes32 keyHash,
        uint64 subId,
        uint16 requestConfirmations,
        uint32 callbackGasLimit,
        uint32 numWords
    ) external override vrfLock returns (uint256) {
        // Input validation using the subscription storage.
        if (s_subscriptionConfigs[subId].owner == address(0)) {
            revert InvalidSubscription();
        }
        // Its important to ensure that the consumer is in fact who they say they
        // are, otherwise they could use someone else's subscription balance.
        // A nonce of 0 indicates consumer is not allocated to the sub.
        uint64 currentNonce = s_consumers[msg.sender][subId];
        if (currentNonce == 0) {
            revert InvalidConsumer(subId, msg.sender);
        }
        // Input validation using the config storage word.
        if (
            requestConfirmations < s_config.minimumRequestConfirmations || requestConfirmations > MAX_REQUEST_CONFIRMATIONS
        ) {
            revert InvalidRequestConfirmations(
                requestConfirmations,
                s_config.minimumRequestConfirmations,
                MAX_REQUEST_CONFIRMATIONS
            );
        }
        // No lower bound on the requested gas limit. A user could request 0
        // and they would simply be billed for the proof verification and wouldn't be
        // able to do anything with the random value.
        if (callbackGasLimit > s_config.maxGasLimit) {
            revert GasLimitTooBig(callbackGasLimit, s_config.maxGasLimit);
        }
        if (numWords > s_config.maxNumWords) {
            revert NumWordsTooBig(numWords, s_config.maxNumWords);
        }
        // Note we do not check whether the keyHash is valid to save gas.
        // The consequence for users is that they can send requests
        // for invalid keyHashes which will simply not be fulfilled.
        uint64 nonce = currentNonce + 1;
        (uint256 requestId, uint256 preSeed) = computeRequestId(keyHash, msg.sender, subId, nonce);

        s_requestCommitments[requestId] = keccak256(
            abi.encode(requestId, block.number, subId, callbackGasLimit, numWords, msg.sender)
        );
        emit RandomWordsRequested(
            keyHash,
            requestId,
            preSeed,
            subId,
            requestConfirmations,
            callbackGasLimit,
            numWords,
            msg.sender
        );
        s_consumers[msg.sender][subId] = nonce;

        return requestId;
    }

    /**
    * @notice Get request commitment
    * @param requestId id of request
    * @dev used to determine if a request is fulfilled or not
    */
    function getCommitment(uint256 requestId) external view returns (bytes32) {
        return s_requestCommitments[requestId];
    }

    function computeRequestId(
        bytes32 keyHash,
        address sender,
        uint64 subId,
        uint64 nonce
    ) private pure returns (uint256, uint256) {
        uint256 preSeed = uint256(keccak256(abi.encode(keyHash, sender, subId, nonce)));
        return (uint256(keccak256(abi.encode(keyHash, preSeed))), preSeed);

    }

    /**
     * @dev calls target address with exactly gasAmount gas and data as calldata
     * or reverts if at least gasAmount gas is not available.
     */
    function callWithExactGas(
        uint256 gasAmount,
        address target,
        bytes memory data
    ) private returns (bool success) {
        // solhint-disable-next-line no-inline-assembly
        assembly {
            let g := gas()
        // Compute g -= GAS_FOR_CALL_EXACT_CHECK and check for underflow
        // The gas actually passed to the callee is min(gasAmount, 63//64*gas available).
        // We want to ensure that we revert if gasAmount >  63//64*gas available
        // as we do not want to provide them with less, however that check itself costs
        // gas.  GAS_FOR_CALL_EXACT_CHECK ensures we have at least enough gas to be able
        // to revert if gasAmount >  63//64*gas available.
            if lt(g, GAS_FOR_CALL_EXACT_CHECK) {
                revert(0, 0)
            }
            g := sub(g, GAS_FOR_CALL_EXACT_CHECK)
        // if g - g//64 <= gasAmount, revert
        // (we subtract g//64 because of EIP-150)
            if iszero(gt(sub(g, div(g, 64)), gasAmount)) {
                revert(0, 0)
            }
        // solidity calls check that a contract actually exists at the destination, so we do the same
            if iszero(extcodesize(target)) {
                revert(0, 0)
            }
        // call and return whether we succeeded. ignore return data
        // call(gas,addr,value,argsOffset,argsLength,retOffset,retLength)
            success := call(gasAmount, target, 0, add(data, 0x20), mload(data), 0, 0)
        }
        return success;
    }

    function getRandomnessFromProof(Proof memory proof, RequestCommitment memory rc)
    private
    view
    returns (
        uint256 requestId,
        uint256 randomness
    )
    {
        bytes32 keyHash = hashOfKey(proof.pk);
        // Only registered proving keys are permitted.
        bool flag = false;
        for (uint256 i = 0; i < s_provingKeyHashes.length; i++) {
            if (s_provingKeyHashes[i] == keyHash) {
                flag = true;
            }
        }
        if (!flag) {
            revert NoSuchProvingKey(keyHash);
        }
        requestId = uint256(keccak256(abi.encode(keyHash, proof.seed)));
        bytes32 commitment = s_requestCommitments[requestId];
        if (commitment == 0) {
            revert NoCorrespondingRequest();
        }
        if (
            commitment != keccak256(abi.encode(requestId, rc.blockNum, rc.subId, rc.callbackGasLimit, rc.numWords, rc.sender))
        ) {
            revert IncorrectCommitment();
        }

        bytes32 blockHash = blockhash(rc.blockNum);
        if (blockHash == bytes32(0)) {
            blockHash = BLOCKHASH_STORE.getBlockhash(rc.blockNum);
            if (blockHash == bytes32(0)) {
                revert BlockhashNotInStore(rc.blockNum);
            }
        }

        // The seed actually used by the VRF machinery, mixing in the blockhash
        uint256 actualSeed = uint256(keccak256(abi.encodePacked(proof.seed, blockHash)));
        randomness = VRF.randomValueFromVRFProof(proof, actualSeed); // Reverts on failure
    }

    /*
    * @notice Compute fee based on the request count
    * @param reqCount number of requests
    * @return feePPM fee
    */
    function getFeeTier(uint64 reqCount, uint64 subId) public view returns (uint32) {
        if (s_designated_subId_fee[subId] != 0 ){
            return s_designated_subId_fee[subId];
        }
        FeeConfig memory fc = s_feeConfig;
        if (0 <= reqCount && reqCount <= fc.reqsForTier2) {
            return fc.fulfillmentFlatFeeBNBPPMTier1;
        }
        if (fc.reqsForTier2 < reqCount && reqCount <= fc.reqsForTier3) {
            return fc.fulfillmentFlatFeeBNBPPMTier2;
        }
        if (fc.reqsForTier3 < reqCount && reqCount <= fc.reqsForTier4) {
            return fc.fulfillmentFlatFeeBNBPPMTier3;
        }
        if (fc.reqsForTier4 < reqCount && reqCount <= fc.reqsForTier5) {
            return fc.fulfillmentFlatFeeBNBPPMTier4;
        }
        return fc.fulfillmentFlatFeeBNBPPMTier5;
    }

    /**
    * @notice set feeTier by the designated subId
    * @param subId (Unique identification of subscription accounts)
    * @param feeTier, feeTier of the designated subId
    */
    function setDesignatedSubIdFeeTier(uint64 subId, uint32 feeTier) external onlyOwner {
        if (s_subscriptionConfigs[subId].owner == address(0)) {
            revert InvalidSubscription();
        }
        s_designated_subId_fee[subId] = feeTier;
    }

    /**
    * @notice get designated feeTier by subId
    * @param subId (Unique identification of subscription accounts)
    * @return feeTier of the designated subId
    */
    function getDesignatedSubIdFeeTier(uint64 subId) external view returns (uint32) {
        return s_designated_subId_fee[subId];
    }

    /*
     * @notice Fulfill a randomness request
     * @param proof contains the proof and randomness
     * @param rc request commitment pre-image, committed to at request time
     * @return payment amount billed to the subscription
     * @dev simulated offchain to determine if sufficient balance is present to fulfill the request
     */
    function fulfillRandomWords(Proof memory proof, RequestCommitment memory rc) external vrfLock checkNodeAccess(msg.sender) returns (uint96) {
        uint256 startGas = gasleft();
        (uint256 requestId, uint256 randomness) = getRandomnessFromProof(proof, rc);

        uint256[] memory randomWords = new uint256[](rc.numWords);
        for (uint256 i = 0; i < rc.numWords; i++) {
            randomWords[i] = uint256(keccak256(abi.encode(randomness, i)));
        }

        delete s_requestCommitments[requestId];
        VRFConsumerBase v;
        bytes memory resp = abi.encodeWithSelector(v.rawFulfillRandomWords.selector, requestId, randomWords);
        // Call with explicitly the amount of callback gas requested
        // Important to not let them exhaust the gas budget and avoid oracle payment.
        // Do not allow any non-view/non-pure coordinator functions to be called
        // during the consumers callback code via reentrancyLock.
        // Note that callWithExactGas will revert if we do not have sufficient gas
        // to give the callee their requested amount.
        s_config.reentrancyLock = true;
        bool success = callWithExactGas(rc.callbackGasLimit, rc.sender, resp);
        s_config.reentrancyLock = false;

        // Increment the req count for fee tier selection.
        uint64 subId = rc.subId;
        uint64 reqCount = s_subscriptions[subId].reqCount;
        s_subscriptions[subId].reqCount += 1;

        // We want to charge users exactly for how much gas they use in their callback.
        // The gasAfterPaymentCalculation is meant to cover these additional operations where we
        // decrement the subscription balance and increment the oracles withdrawable balance.
        (uint96 payment, uint256 fee) = calculatePaymentAmount(
            startGas,
            s_config.gasAfterPaymentCalculation,
            getFeeTier(reqCount, subId),
            tx.gasprice
        );
        if (s_subscriptions[subId].balance < payment) {
            revert InsufficientBalance(s_subscriptions[subId].balance, payment);
        }
        s_subscriptions[subId].balance -= payment;
        s_withdrawableTokens += payment;
        // Include payment in the event for tracking costs.
        emit RandomWordsFulfilled(requestId, randomness, payment, fee, success);
        return payment;
    }

    // Get the amount of gas used for fulfillment
    function calculatePaymentAmount(
        uint256 startGas,
        uint256 gasAfterPaymentCalculation,
        uint32 fulfillmentFlatFeeBNBPPMTier,
        uint256 weiPerUnitGas
    ) internal view returns (uint96, uint256) {
        uint256 paymentNoFee = weiPerUnitGas * (gasAfterPaymentCalculation + startGas - gasleft());
        uint256 fee = 1e12 * uint256(fulfillmentFlatFeeBNBPPMTier);
        return (uint96(paymentNoFee + fee), fee);
    }

    function getCurrentSubId() external view returns (uint64) {
        return s_currentSubId;
    }

    /**
     * @inheritdoc VRFCoordinatorInterface
     */
    function getSubscription(uint64 subId)
    external
    view
    override
    returns (
        uint96 balance,
        uint64 reqCount,
        address owner,
        address[] memory consumers
    )
    {
        if (s_subscriptionConfigs[subId].owner == address(0)) {
            revert InvalidSubscription();
        }
        return (
        s_subscriptions[subId].balance,
        s_subscriptions[subId].reqCount,
        s_subscriptionConfigs[subId].owner,
        s_subscriptionConfigs[subId].consumers
        );
    }

    /**
     * @inheritdoc VRFCoordinatorInterface
     */
    function createSubscription() external override vrfLock returns (uint64) {
        s_currentSubId++;
        uint64 currentSubId = s_currentSubId;
        address[] memory consumers = new address[](0);
        s_subscriptions[currentSubId] = Subscription({balance: 0, reqCount: 0});
        s_subscriptionConfigs[currentSubId] = SubscriptionConfig({
        owner: msg.sender,
        requestedOwner: address(0),
        consumers: consumers
        });

        emit SubscriptionCreated(currentSubId, msg.sender);
        return currentSubId;
    }

    /**
     * @inheritdoc VRFCoordinatorInterface
     */
    function requestSubscriptionOwnerTransfer(uint64 subId, address newOwner)
    external
    override
    onlySubOwner(subId)
    vrfLock
    {
        // Proposing to address(0) would never be claimable so don't need to check.
        if (s_subscriptionConfigs[subId].requestedOwner != newOwner) {
            s_subscriptionConfigs[subId].requestedOwner = newOwner;
            emit SubscriptionOwnerTransferRequested(subId, msg.sender, newOwner);
        }
    }

    /**
     * @inheritdoc VRFCoordinatorInterface
     */
    function acceptSubscriptionOwnerTransfer(uint64 subId) external override vrfLock {
        if (s_subscriptionConfigs[subId].owner == address(0)) {
            revert InvalidSubscription();
        }
        if (s_subscriptionConfigs[subId].requestedOwner != msg.sender) {
            revert MustBeRequestedOwner(s_subscriptionConfigs[subId].requestedOwner);
        }
        address oldOwner = s_subscriptionConfigs[subId].owner;
        s_subscriptionConfigs[subId].owner = msg.sender;
        s_subscriptionConfigs[subId].requestedOwner = address(0);
        emit SubscriptionOwnerTransferred(subId, oldOwner, msg.sender);
    }

    /**
     * @inheritdoc VRFCoordinatorInterface
     */
    function removeConsumer(uint64 subId, address consumer) external override onlySubOwner(subId) vrfLock {
        if (s_consumers[consumer][subId] == 0) {
            revert InvalidConsumer(subId, consumer);
        }
        // Note bounded by MAX_CONSUMERS
        address[] memory consumers = s_subscriptionConfigs[subId].consumers;
        uint256 lastConsumerIndex = consumers.length - 1;
        for (uint256 i = 0; i < consumers.length; i++) {
            if (consumers[i] == consumer) {
                address last = consumers[lastConsumerIndex];
                // Storage write to preserve last element
                s_subscriptionConfigs[subId].consumers[i] = last;
                // Storage remove last element
                s_subscriptionConfigs[subId].consumers.pop();
                break;
            }
        }
        delete s_consumers[consumer][subId];
        emit SubscriptionConsumerRemoved(subId, consumer);
    }

    /**
     * @inheritdoc VRFCoordinatorInterface
     */
    function addConsumer(uint64 subId, address consumer) external override onlySubOwner(subId) vrfLock {
        // Already maxed, cannot add any more consumers.
        if (s_subscriptionConfigs[subId].consumers.length == MAX_CONSUMERS) {
            revert TooManyConsumers();
        }
        if (s_consumers[consumer][subId] != 0) {
            // Idempotence - do nothing if already added.
            // Ensures uniqueness in s_subscriptions[subId].consumers.
            return;
        }
        // Initialize the nonce to 1, indicating the consumer is allocated.
        s_consumers[consumer][subId] = 1;
        s_subscriptionConfigs[subId].consumers.push(consumer);

        emit SubscriptionConsumerAdded(subId, consumer);
    }

    /**
     * @inheritdoc VRFCoordinatorInterface
     */
    function cancelSubscription(uint64 subId, address to) external override onlySubOwner(subId) {
        if (pendingRequestExists(subId)) {
            revert PendingRequestExists();
        }
        cancelSubscriptionHelper(subId, to);
    }

    function cancelSubscriptionHelper(uint64 subId, address to) private vrfLock {
        SubscriptionConfig memory subConfig = s_subscriptionConfigs[subId];
        Subscription memory sub = s_subscriptions[subId];
        uint96 balance = sub.balance;
        // Note bounded by MAX_CONSUMERS;
        // If no consumers, does nothing.
        for (uint256 i = 0; i < subConfig.consumers.length; i++) {
            delete s_consumers[subConfig.consumers[i]][subId];
        }
        delete s_subscriptionConfigs[subId];
        delete s_subscriptions[subId];
        s_totalBalance -= balance;
        (bool success,) = payable(to).call{value: balance}("");
        if (!success) {
            revert InsufficientBalance(s_totalBalance, balance);
        }
        emit SubscriptionCanceled(subId, to, balance);
    }

    /**
    * @notice withdrawal of available balance
    * @param amount, how much to withdraw
    */
    function oracleWithdraw(uint96 amount) external vrfLock {
        if (s_withdrawableTokens < amount) {
            revert InsufficientBalance(s_withdrawableTokens, amount);
        }
        if (s_withdrawableTokensAddress == address(0)) {
            revert InvalidAddress();
        }
        s_withdrawableTokens -= amount;
        s_totalBalance -= amount;
        (bool success,) = s_withdrawableTokensAddress.call{value: amount}("");
        if (!success) {
            revert InsufficientBalance(s_withdrawableTokens, amount);
        }
    }

    /**
    * @notice set wallet address for withdraw
    * @param recipient, wallet address
    */
    function updateAddress(address recipient) external onlyOwner{
        if (recipient == address(0)) {
            revert InvalidAddress();
        }
        s_withdrawableTokensAddress = recipient;
        emit s_withdrawableTokensAddressChange(s_withdrawableTokensAddress, recipient);
    }

    function deposit(uint64 subId) external payable {
        require(msg.value != 0, "ia");
        if (s_subscriptionConfigs[subId].owner == address(0)) {
            revert InvalidSubscription();
        }
        // We do not check that the msg.sender is the subscription owner,
        // anyone can fund a subscription.
        uint256 oldBalance = s_subscriptions[subId].balance;
        uint96 afterDepositAmount = s_subscriptions[subId].balance + uint96(msg.value);
        if(s_config.subAccountMaxDeposit < afterDepositAmount){
            revert ExceedingTheMaximumRechargeLimit(s_config.subAccountMaxDeposit, s_subscriptions[subId].balance, uint96(msg.value));
        }
        s_subscriptions[subId].balance += uint96(msg.value);
        s_totalBalance += uint96(msg.value);
        emit SubscriptionFunded(subId, oldBalance, oldBalance + msg.value);
    }

    /**
    * @inheritdoc VRFCoordinatorInterface
    * @dev Looping is bounded to MAX_CONSUMERS*(number of keyhashes).
    * @dev Used to disable subscription canceling while outstanding request are present.
    */
    function pendingRequestExists(uint64 subId) public view override returns (bool) {
        SubscriptionConfig memory subConfig = s_subscriptionConfigs[subId];
        for (uint256 i = 0; i < subConfig.consumers.length; i++) {
            for (uint256 j = 0; j < s_provingKeyHashes.length; j++) {
                (uint256 reqId, ) = computeRequestId(
                    s_provingKeyHashes[j],
                    subConfig.consumers[i],
                    subId,
                    s_consumers[subConfig.consumers[i]][subId]
                );
                if (s_requestCommitments[reqId] != 0) {
                    return true;
                }
            }
        }
        return false;
    }

    /**
    * @notice set node whitelist switch status, if open, will only the nodes of whitelist can call the "fulfillRandomWords"
    */
    function setNodesWhiteListSwitchStatus(bool status) external onlyOwner {
        nodesWhiteListSwitchStatus = status;
    }

    function getNodesWhiteListSwitchStatus() external view returns (bool) {
        return nodesWhiteListSwitchStatus;
    }

    /**
    * @notice disable designated wallet address in the nodesWhiteList
    * @param node, wallet address
    */
    function disableNodeAccess(address node) external onlyOwner {
        if (nodesWhiteListConfig[node]) {
            nodesWhiteListConfig[node] = false;
            emit NodeAccessDisabled(node);
        }
    }

    /**
    * @notice add designated wallet address to the nodesWhiteList
    * @param node, wallet address
    */
    function addNodeAccess(address node) external onlyOwner {
        if (!nodesWhiteListConfig[node]) {
            nodesWhiteListConfig[node] = true;
            emit NodeAccessAdd(node);
        }
    }

    /**
    * @notice return designated wallet address access status
    */
    function getNodeAccessStatus(address node) external view returns (bool) {
        return nodesWhiteListConfig[node];
    }

    /**
    * @notice check if node has access
    */
    modifier checkNodeAccess(address node) {
        if(nodesWhiteListSwitchStatus && !nodesWhiteListConfig[node]){
            revert NoAuthFulfillRandomWords(node);
        }
        _;
    }

    modifier onlySubOwner(uint64 subId) {
        address owner = s_subscriptionConfigs[subId].owner;
        if (owner == address(0)) {
            revert InvalidSubscription();
        }
        if (msg.sender != owner) {
            revert MustBeSubOwner(owner);
        }
        _;
    }

    modifier vrfLock() {
        if (s_config.reentrancyLock) {
            revert Reentrant();
        }
        _;
    }

    /**
    * @notice The type and version of this contract
    * @return Type and version string
    */
    function typeAndVersion() external pure virtual override returns (string memory) {
        return "VRFCoordinatorV2 1.0.0";
    }
}
