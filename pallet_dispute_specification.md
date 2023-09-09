# Pallet-Dispute: Specification

## Overview

The Substrate Dispute Resolution Pallet is a blockchain-based module designed to facilitate efficient dispute resolution within a Substrate-based blockchain network. This pallet is intended to be a core component of your blockchain application, enabling participants to raise disputes, engage impartial jurors, and ensure transparent and fair resolution.

## Key Components/Actors

1. **Dispute Raiser**: Person who raise the dispute. Could be a brief/project owner or freelancer

2. **Jury**: A pool of  participants selected to vote on dispute resolutions from fellowship randomly. Typically, 7 to 9 fellowship members are randomly chosen for each dispute.

3. **Pallet-Dispute**: pallet to handle the dispute resolution.
   
4. **Pallet-Refund**: Separate pallet for handle the claims like refunding the amount upon dispute resolution

## Process Flow

### 1. Dispute Initiation

- Dispute raisers initiate disputes by submitting information like the dispute key(which is the project_id),reason for dispute

### 2. Jury Selection

- The system selects a panel of random selected juries, typically ranging from 7 to 9. The jury will be responsible for voting their decision on the dispute raised

### 3. Voting Period

- Upon the raising of the dispute, the jury members will be notified to begin voting and 2 weeks of timing will be allotted
- During a voting period, each jury members independently casts their vote, expressing their opinion on the dispute's resolution.
- The selected jury members need to cast their vote during the voting period or they will be entitled for slashing

### 4. Changing the votes

- The jury can change their votes during the voting period

### 5. Refund or Resolution

- If the majority(51%) of jury members vote in favor of the dispute raiser, the refund-pallet will be invoked to process the refund.
- In cases where the dispute raiser is not the majority, the dispute will be cancelled.
