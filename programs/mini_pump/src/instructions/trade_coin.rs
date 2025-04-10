use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, token::{transfer_checked, TransferChecked}, token_interface::{mint_to, Mint, MintTo, TokenAccount, TokenInterface}
};
use anchor_lang::system_program::{transfer, Transfer};

use crate::state::BondingCurve;
use crate::state::GlobalState;


#[derive(Accounts)]
pub struct TradeCoin<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = token_mint,
        associated_token::authority = buyer,
    )]
    pub buyer_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        seeds = ["bonding_curve_sol_escrow".as_bytes(), bonding_curve.key().as_ref()],
        bump,
    )]
    pub sol_escrow: SystemAccount<'info>,

    #[account(
        mut,
        seeds = ["bonding_curve".as_bytes(), bonding_curve.key().as_ref()],
        bump = bonding_curve.bump,
    )]
    pub bonding_curve: Account<'info, BondingCurve>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = bonding_curve,
    )]
    pub bonding_curve_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = ["global_state".as_bytes()],
        bump = global_state.bump,
    )]
    pub global_state: Account<'info, GlobalState>,

    pub token_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    
    pub system_program: Program<'info, System>,

}

impl<'info> TradeCoin<'info> {
    pub fn buy_token(&mut self, sol_amount: u64,) -> Result<()> {
       
        if !self.bonding_curve.is_active {
            return Err(MiniPumpError::BondingCurveNotActive.into());
        }


        let transfer_accounts = Transfer {
            from: self.buyer.to_account_info(),
            to: self.sol_escrow.to_account_info(),
        };

        let transfer_ctx = CpiContext::new(self.system_program.to_account_info(), transfer_accounts);

        transfer(transfer_ctx, sol_amount)?;

        // sol received now trasnfer out the tokens 
        // calculate the tokens to send out 
        let mut token_out = self.calculate_token_for_sol(sol_amount)?;

      

        let bonding_curve: &mut Account<'info, BondingCurve> =  &mut self.bonding_curve;

        // NOTE: This is actually a wrong approach! We need to calculate by the formula
        // how much SOL they should give for the remaining token_out.
        // 
        // HOMEWORK for Turbine attendees: Figure out how to properly calculate this!
        // The current implementation has a critical flaw - if only 1 token is left until
        // the 800 million limit, it will take all the SOL amount and just give back 1 token,
        // which is extremely unfair to the user.
        //
        // The correct approach would be to:
        // 1. Check if we're hitting the limit
        // 2. Calculate how much SOL is needed for the actual tokens being purchased
        // 3. Refund the excess SOL to the buyer
        if bonding_curve.tokens_sold + token_out > 800_000_000_000 {
            token_out = 800_000_000_000 - bonding_curve.tokens_sold;
            bonding_curve.is_active = false;
        }


        let seeds = &[
            "bonding_curve".as_bytes(),
            &bonding_curve.key().to_bytes(),
            &[bonding_curve.bump],
        ];
        
        let signer_seeds = &[&seeds[..]];

        let accounts = TransferChecked{
            from: self.bonding_curve_token_account.to_account_info(),
            to: self.buyer_token_account.to_account_info(),
            mint: self.token_mint.to_account_info(),
            authority: bonding_curve.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), accounts, signer_seeds);

        transfer_checked(cpi_ctx, token_out , self.token_mint.decimals)?;

        bonding_curve.virtual_token_liquidity = bonding_curve.virtual_token_liquidity.checked_sub(token_out).ok_or(MiniPumpError::InsufficientTokenBalance)?;
        bonding_curve.virtual_sol_liquidity = bonding_curve.virtual_sol_liquidity.checked_add(sol_amount).ok_or(MiniPumpError::ArithmeticOverflow)?;
        bonding_curve.tokens_sold = bonding_curve.tokens_sold.checked_add(token_out).ok_or(MiniPumpError::ArithmeticOverflow)?;


        Ok(())
    }

    pub fn sell_token(&mut self, token_amount: u64,) -> Result<()> {
        // now for selling first we transfer in the tokens from the caller. 
        if !self.bonding_curve.is_active {
            return Err(MiniPumpError::BondingCurveNotActive.into());
        }

        let accounts = TransferChecked{
            from: self.buyer_token_account.to_account_info(),
            to: self.bonding_curve_token_account.to_account_info(),
            mint: self.token_mint.to_account_info(),
            authority: self.buyer.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), accounts);

        transfer_checked(cpi_ctx, token_amount, self.token_mint.decimals)?;

        let sol_amount = self.calculate_sol_for_token(token_amount)?;


        let bonding_curve = &mut self.bonding_curve;


        let transfer_accounts = Transfer {
            from: self.bonding_curve_token_account.to_account_info(),
            to: self.sol_escrow.to_account_info(),
        };

        let seeds = &[
            "bonding_curve".as_bytes(),
            &bonding_curve.key().to_bytes(),
            &[bonding_curve.bump],
        ];

        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(self.system_program.to_account_info(), transfer_accounts, signer_seeds);

        transfer(cpi_ctx, sol_amount)?;

        bonding_curve.virtual_token_liquidity = bonding_curve.virtual_token_liquidity.checked_sub(token_amount).ok_or(MiniPumpError::InsufficientTokenBalance)?;
        bonding_curve.virtual_sol_liquidity = bonding_curve.virtual_sol_liquidity.checked_add(sol_amount).ok_or(MiniPumpError::ArithmeticOverflow)?;
        bonding_curve.tokens_sold = bonding_curve.tokens_sold.checked_add(token_amount).ok_or(MiniPumpError::ArithmeticOverflow)?;

      
        
        Ok(())
    }

    


    /// Calculates the amount of tokens to be received for a given SOL amount
    /// 
    /// This function implements a modified constant product formula for bonding curves:
    /// 
    /// The formula is derived from the constant product AMM formula: x * y = k
    /// where x = virtual_sol_liquidity and y = virtual_token_liquidity
    /// 
    /// For a bonding curve with virtual liquidity, we use:
    /// virtual_sol_liquidity * virtual_token_liquidity = constant (k)
    /// 
    /// When a user buys tokens with SOL:
    /// (virtual_sol_liquidity + sol_amount) * new_token_supply = k
    /// 
    /// Since k = virtual_sol_liquidity * virtual_token_liquidity, we can substitute:
    /// (virtual_sol_liquidity + sol_amount) * new_token_supply = virtual_sol_liquidity * virtual_token_liquidity
    /// 
    /// Solving for new_token_supply:
    /// new_token_supply = (virtual_sol_liquidity * virtual_token_liquidity) / (virtual_sol_liquidity + sol_amount)
    /// 
    /// The tokens sent to the user are:
    /// token_amount = virtual_token_liquidity - new_token_supply
    /// 
    /// This creates a price curve that increases as more tokens are purchased,
    /// since the virtual_token_liquidity decreases with each purchase while
    /// virtual_sol_liquidity increases, making each subsequent token more expensive.
    /// 
    /// Price curve visualization:
    /// 
    ///  Price
    ///    ^
    ///    |                                  /|
    ///    |                                /
    ///    |                              /
    ///    |                            /
    ///    |                         /
    ///    |                      /
    ///    |                   /
    ///    |               /
    ///    |          _/
    ///    |____----
    ///    +------------------------------------> Tokens Sold
    ///
    /// As more tokens are sold, the price increases exponentially due to the
    /// constant product formula, creating a natural price discovery mechanism.
    pub fn calculate_token_for_sol(&self, sol_amount: u64) -> Result<u64> {
        let bonding_curve = &self.bonding_curve;
        
        // Calculate new token supply after adding SOL to the virtual liquidity
        // Formula: new_token_supply = virtual_sol_liquidity * virtual_token_liquidity / (virtual_sol_liquidity + sol_amount)
        let new_token_supply = bonding_curve.virtual_sol_liquidity * bonding_curve.virtual_token_liquidity / (bonding_curve.virtual_sol_liquidity + sol_amount);
        
        // The tokens to send out are the difference between current virtual token liquidity and new token supply
        let token_amount = bonding_curve.virtual_token_liquidity - new_token_supply;
        
        Ok(token_amount)
    }

    /// Calculates the amount of SOL to be received for a given token amount
    /// 
    /// This function implements the inverse of the modified constant product formula:
    /// 
    /// Starting with the constant product formula: x * y = k
    /// where x = virtual_sol_liquidity and y = virtual_token_liquidity
    /// 
    /// When a user sells tokens:
    /// new_sol_supply * (virtual_token_liquidity + token_amount) = k
    /// 
    /// Since k = virtual_sol_liquidity * virtual_token_liquidity, we can substitute:
    /// new_sol_supply * (virtual_token_liquidity + token_amount) = virtual_sol_liquidity * virtual_token_liquidity
    /// 
    /// Solving for new_sol_supply:
    /// new_sol_supply = (virtual_sol_liquidity * virtual_token_liquidity) / (virtual_token_liquidity + token_amount)
    /// 
    /// The SOL sent to the user is:
    /// sol_amount = virtual_sol_liquidity - new_sol_supply
    /// 
    /// This creates a price curve that decreases as more tokens are sold,
    /// since the virtual_token_liquidity increases with each sale while
    /// virtual_sol_liquidity decreases, making each subsequent token less valuable.
    /// 
    /// Price curve for selling visualization:
    /// 
    ///  SOL Received
    ///    ^
    ///    |\
    ///    | \
    ///    |  \
    ///    |   \
    ///    |    \
    ///    |     \
    ///    |      \
    ///    |       \
    ///    |        \__
    ///    |           ----___________
    ///    +------------------------------------> Tokens Sold Back
    ///
    /// When selling tokens back to the curve, the amount of SOL received
    /// decreases as more tokens are sold, following the inverse of the
    /// bonding curve formula. This creates a natural disincentive for
    /// large sell-offs and helps stabilize the token price.
    pub fn calculate_sol_for_token(&self, token_amount: u64) -> Result<u64> {
        let bonding_curve = &self.bonding_curve;
        
        // Calculate new SOL supply after adding tokens to the virtual liquidity
        // Formula: new_sol_supply = virtual_sol_liquidity * virtual_token_liquidity / (virtual_token_liquidity + token_amount)
        let new_sol_supply = bonding_curve.virtual_sol_liquidity * (bonding_curve.virtual_token_liquidity) / (bonding_curve.virtual_token_liquidity + token_amount);
        
        // The SOL to send out is the difference between current virtual SOL liquidity and new SOL supply
        let sol_amount = bonding_curve.virtual_sol_liquidity - new_sol_supply;
        
        Ok(sol_amount)
    }

}


#[error_code]
pub enum MiniPumpError {
    #[msg("Insufficient token balance")]
    InsufficientTokenBalance,
    #[msg("Insufficient SOL balance")]
    InsufficientSolBalance,
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
    #[msg("Invalid token amount")]
    InvalidTokenAmount,
    #[msg("Invalid SOL amount")]
    InvalidSolAmount,
    #[msg("Calculation error")]
    CalculationError,
    #[msg("Token sold limit reached")]
    TokenSoldLimitReached,
    #[msg("Bonding curve not active")]
    BondingCurveNotActive,
}
