# Acai

![Acai Logo](acai.png)

These are the sources deployed at [4, 616361690].

Acai is a modification of the "free-mint.wasm" smart contract for the Alkanes protocol, designed to enhance minting reliability and protect against RBF bots. The contract operates within Bitcoin Core standards while creating a strong economic incentive for using the Rebar Shield private mempool.

## Problem and Solution

Traditional minting transactions in Bitcoin are vulnerable to MEV and frontrun attacks when sent through the public mempool. Acai addresses this by creating conditions where:

- In the public mempool, the probability of successful minting is only 14-20%
- When using Rebar Shield, 100% success is guaranteed without using non-standard transactions

## How It Works

The contract analyzes the scriptSig of the block's coinbase input - a field where miners can write arbitrary data when creating a block. If this data contains a substring matching one of Rebar's partner pools (e.g., AntPool), the mint is considered successful. Otherwise, the operation is rejected.

This ensures:
- Operations are only executed in blocks mined by approved pools
- Technical compatibility with the public mempool, but practical inapplicability
- Rebar Shield becomes the most efficient way to interact with the contract

## Results

The Acai contract is currently being battle-tested in the [fartane.com](http://fartane.com/) application, created to support the interaction between Alkanes and Rebar. From May 14th to May 18th, over 300 successful minting transactions were executed through Rebar Shield.

## Potential Applications

Beyond token minting, Acai's operating principle can be applied to other MEV-risk operations:
- Order placement in onchain order books
- Time-sensitive event triggering (auctions, lotteries)
- Reward or drop distribution
- Key actions in lending protocols
