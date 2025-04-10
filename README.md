# Mini Pump

A bonding curve token launch protocol for Solana that enables fair price discovery and seamless DEX migration.

## Overview

Mini Pump is a minified version of [Pump Science](https://github.com/code-423n4/2025-01-pump-science) created for educational purposes to help developers understand how to implement bonding curve mechanisms in their Solana programs.

## Bonding Curve Mechanism

Mini Pump implements a modified constant product formula for its bonding curve:

- When users buy tokens with SOL, the price increases as more tokens are purchased
- When users sell tokens back, they receive SOL based on the current position on the curve
- The protocol uses virtual liquidity to create a smooth price curve

### Buy Formula
```
token_amount = virtual_token_liquidity - (virtual_sol_liquidity * virtual_token_liquidity) / (virtual_sol_liquidity + sol_amount)
```

### Sell Formula
```
sol_amount = virtual_sol_liquidity - (virtual_sol_liquidity * virtual_token_liquidity) / (virtual_token_liquidity + token_amount)
```

## Key Features

- **Fair Launch**: Tokens start at a low price and increase as more are purchased
- **Token Supply Cap**: Maximum of 800 million tokens can be sold
- **Automatic Deactivation**: Bonding curve automatically deactivates when token limit is reached
- **SOL Escrow**: All SOL is held in a secure escrow account
- **Withdrawal Mechanism**: Project owners can withdraw accumulated SOL

## Program Entry Points

The protocol exposes the following instruction handlers:

1. `init_protocol` - Initialize the global state of the protocol
2. `launch_coin` - Create a new bonding curve for a token
3. `trade_coin` - Buy or sell tokens using the bonding curve
4. `withdraw_funds` - Allow project owners to withdraw SOL from the escrow

## Usage

To interact with the Mini Pump protocol:

1. Initialize the protocol (admin only)
2. Launch a new token with a bonding curve
3. Users can buy tokens, which increases the price according to the curve
4. Users can sell tokens back to the protocol at any time
5. When the token cap is reached, the bonding curve deactivates

## Development Notes

The current implementation has a known issue in the `buy_token` function that needs to be addressed:

- When approaching the token supply limit, the function doesn't properly calculate the correct amount of SOL needed for the remaining tokens
- This can result in users paying the full SOL amount but receiving fewer tokens than expected
- A proper implementation would calculate the exact SOL needed and refund any excess



## Security Considerations

⚠️ **IMPORTANT DISCLAIMER** ⚠️

This code is provided strictly for **educational purposes only** and should NOT be used in production environments. Please be aware of the following:

- This implementation has not undergone professional security audits
- The code may contain bugs, vulnerabilities, or edge cases that could lead to loss of funds
- While tests will be added over time, the test coverage is currently incomplete
- The mathematical models have not been extensively validated in real-world scenarios

Mini Pump is intended as a learning resource for developers interested in understanding bonding curve mechanisms on Solana. It serves as inspiration and reference material for those looking to implement similar systems.

If you're considering implementing a bonding curve mechanism for actual token launches:
- Engage professional auditors to review your code
- Conduct extensive testing across various scenarios
- Consider all edge cases and attack vectors
- Implement proper access controls and security measures
- Thoroughly validate the mathematical models

The authors of Mini Pump assume no liability for any losses incurred through the use of this code in production environments.