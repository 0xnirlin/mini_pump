use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenInterface, TokenAccount, TransferChecked, transfer_checked}
};
use anchor_lang::system_program::{transfer, Transfer};

use crate::state::global_state::GlobalState;
use crate::state::bonding_curve::BondingCurve;

/// # Withdraw Funds Instruction
///
/// This instruction enables the protocol owner to withdraw all SOL and remaining tokens 
/// (up to 200 million) from a deactivated bonding curve. The primary purpose is to migrate 
/// these assets to a decentralized exchange like Raydium to establish a liquidity pool.
///
/// ## Purpose and Lifecycle
/// 1. Initial phase: Token trading occurs through the bonding curve mechanism
/// 2. Transition phase: Once the bonding curve is deactivated (either by reaching token limit or manual deactivation)
/// 3. Migration phase: This instruction withdraws all assets to create liquidity on a DEX
/// 4. Final phase: Trading continues on the DEX with market-based price discovery
///
/// ## Liquidity Migration Benefits
/// - Provides continued trading after the bonding curve phase
/// - Establishes market-driven price discovery on a DEX
/// - Increases token accessibility and trading options for holders
/// - Creates a sustainable long-term trading environment
#[derive(Accounts)]
pub struct WithdrawFunds<'info> {
    /// The protocol owner who will receive the withdrawn assets
    /// This account will be responsible for creating the DEX liquidity pool
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The global state account containing protocol ownership information
    #[account(
        seeds = ["global_state".as_bytes()],
        bump = global_state.bump,
    )]
    pub global_state: Account<'info, GlobalState>,

    /// The SOL escrow account that holds all SOL collected during bonding curve operations
    /// All SOL will be withdrawn to create the SOL side of the DEX liquidity pool
    #[account(
        seeds = ["bonding_curve_sol_escrow".as_bytes(), bonding_curve.key().as_ref()],
        bump,
    )]
    pub sol_escrow: SystemAccount<'info>,

    /// The bonding curve account that must be inactive before migration
    /// Contains state information about the token's bonding curve
    #[account(
        seeds = ["bonding_curve".as_bytes(), token_mint.key().as_ref()],
        bump = bonding_curve.bump,
    )]
    pub bonding_curve: Account<'info, BondingCurve>,

    /// The token account owned by the bonding curve
    /// Contains the remaining tokens (up to 200 million) that will form the token side of the DEX liquidity pool
    #[account(
        seeds = ["bonding_curve_token_account".as_bytes(), bonding_curve.key().as_ref()],
        bump,
    )]
    pub bonding_curve_token_account: InterfaceAccount<'info, TokenAccount>,

    /// The owner's token account that will temporarily hold the tokens before DEX pool creation
    #[account(
        mut, 
        associated_token::mint = token_mint,
        associated_token::authority = payer,
    )]
    pub payer_token_account: InterfaceAccount<'info, TokenAccount>,

    /// The mint of the token that will be paired with SOL in the DEX liquidity pool
    pub token_mint: InterfaceAccount<'info, Mint>,

    /// The token program used for token transfers
    pub token_program: Interface<'info, TokenInterface>,

    /// The associated token program for token account validation
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// The system program for SOL transfers
    pub system_program: Program<'info, System>,
}

impl<'info> WithdrawFunds<'info> {
    /// Withdraws all SOL and remaining tokens from the bonding curve for DEX liquidity migration
    ///
    /// This function performs the complete asset withdrawal needed before creating a DEX liquidity pool:
    /// 1. Transfers all accumulated SOL from the bonding curve escrow to the owner
    /// 2. Transfers all remaining tokens (up to 200 million) from the bonding curve to the owner
    ///
    /// After this function executes successfully, the owner should:
    /// - Create a liquidity pool on Raydium or another Solana DEX
    /// - Deposit the withdrawn SOL and tokens into the pool
    /// - Enable market-based trading for the token
    pub fn withdraw_funds(&mut self) -> Result<()> {
        // Verify the caller is the protocol owner with migration authority
        require!(self.payer.key() == self.global_state.owner, MiniPumpError::NotOwner);
        
        // Ensure there is SOL available to withdraw for the DEX liquidity pool
        require!(self.sol_escrow.lamports() > 0, MiniPumpError::InsufficientSolBalance);
        
        // Confirm the bonding curve is deactivated before migration
        // This prevents premature liquidity withdrawal that could harm traders
        require!(!self.bonding_curve.is_active, MiniPumpError::BondingCurveActive);

        // Step 1: Transfer all SOL from the escrow to the owner for DEX liquidity
        let cpi_ctx = CpiContext::new(self.system_program.to_account_info(), Transfer {
            from: self.sol_escrow.to_account_info(),
            to: self.payer.to_account_info(),
        });

        transfer(cpi_ctx, self.sol_escrow.lamports())?;

        // Step 2: Transfer all remaining tokens to the owner for DEX liquidity
        // These tokens (up to 200 million) will form the token side of the DEX pool
        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), TransferChecked {
            from: self.bonding_curve_token_account.to_account_info(),
            to: self.payer_token_account.to_account_info(),
            mint: self.token_mint.to_account_info(),
            authority: self.bonding_curve.to_account_info(),
        });

        // Calculate the remaining tokens to transfer
        // - virtual_token_liquidity is 1 billion (total supply)
        // - tokens_sold is 800 million (already sold through bonding curve)
        // - The remaining 200 million tokens are transferred to the owner for DEX liquidity
        transfer_checked(cpi_ctx, self.bonding_curve.virtual_token_liquidity - self.bonding_curve.tokens_sold, self.token_mint.decimals)?;
        Ok(())
    }
}

/// Error codes specific to the DEX migration withdrawal process
#[error_code]
pub enum MiniPumpError {
    /// Returned when someone other than the protocol owner attempts migration
    #[msg("Not owner")]
    NotOwner,
    
    /// Returned when there is insufficient SOL for meaningful DEX liquidity
    #[msg("Insufficient SOL balance")]
    InsufficientSolBalance,
    
    /// Returned when attempting to migrate from an active bonding curve
    /// Migration must only occur after the bonding curve phase is complete
    #[msg("Bonding curve is active")]
    BondingCurveActive,
}
