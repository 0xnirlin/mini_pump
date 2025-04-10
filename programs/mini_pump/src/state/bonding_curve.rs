use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct BondingCurve {
    // first thing we need is virtual_sol_lqiiodituy
    pub virtual_sol_liquidity: u64,
    pub virtual_token_liquidity: u64,
    pub tokens_sold: u64,
    pub token_mint: Pubkey,
    pub is_active: bool,
    pub bump: u8,
}

// the above will define the curve
// apart from these other things we have are the total tokens to mint which will be equal to 