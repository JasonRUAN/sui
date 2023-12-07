// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

interface IBridge {
	// ENUMS

	// Define an enum for the chain IDs
	enum ChainID {
		SUI_CHAIN,
		BTC_CHAIN,
		ETH_CHAIN,
		TMP_CHAIN
	}

	// Define an enum for the token IDs
	enum TokenID {
		SUI_TOKEN,
		BTC_TOKEN,
		ETH_TOKEN,
		USDC_TOKEN,
		USDT_TOKEN
	}

	// STRUCTS

	struct Erc20Transfer {
		bytes32 dataDigest;
		uint256 amount;
		address from;
		address to;
	}

	struct BridgeMessage {
		// 0: token , 1: object ? TBD
		uint8 messageType;
		uint8 version;
		ChainID sourceChain;
		uint64 bridgeSeqNum;
		address senderAddress;
		uint8 targetChain;
		address targetAddress;
		// bytes payload;
	}

	// A struct to represent a validator
	struct Validator {
		address addr; // The address of the validator
		uint256 weight; // The weight of the validator
	}

	struct ApprovedBridgeMessage {
		BridgeMessage message;
		uint64 approvedEpoch;
		bytes[] signatures;
	}

	struct BridgeMessageKey {
		uint8 sourceChain;
		uint64 bridgeSeqNum;
	}

	// EVENTS

	// Event to emit when a transfer is initiated
	event ValidatorAdded(
		address addr, // The address of the validator
		uint256 weight // The weight of the validator
	);

	// Event to emit when a transfer is initiated
	event TransferInitiated(
		address indexed sender,
		address indexed recipient,
		uint256 amount,
		uint256 nonce
	);

	// Event to emit when a transfer is completed
	event TransferCompleted(
		address indexed sender,
		address indexed recipient,
		uint256 amount,
		uint256 nonce
	);

	event ContractUpgraded(address indexed oldContract, address indexed newContract);

	event TransferRedeemed(
		uint16 indexed emitterChainId,
		bytes32 indexed emitterAddress,
		uint64 indexed sequence
	);

	event BridgeEvent(BridgeMessage message, bytes message_bytes);
}
