use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct GlobalState {
    pub owner: Pubkey,
    pub tokens_to_sell: Pubkey, 
    pub total_tokens_to_mint: u64,
    pub virtual_sol_liquidity: u64,
    pub virtual_token_liquidity: u64,
    pub bump: u8,
}


// token_to_sell will be 800 million
// total tokens to mint will be 1 billion - remaining 200 will go to the migrator to create the lqiudity on the dex. 
