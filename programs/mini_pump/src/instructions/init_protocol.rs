use anchor_lang::prelude::*;
use crate::state::global_state::GlobalState;
#[derive(Accounts)]
pub struct InitProtocol<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(init,
    payer = payer,
    space = 8 + GlobalState::INIT_SPACE,
    seeds = ["global_state".as_bytes()],
    bump,
    )]
    pub global_state: Account<'info, GlobalState>,
    pub system_program: Program<'info, System>,
}


impl<'info> InitProtocol<'info> {
    pub fn init_protocol(&mut self, total_tokens_to_mint: u64, virtual_sol_liquidity: u64, virtual_token_liquidity: u64, tokens_to_sell: Pubkey, bumps: InitProtocolBumps) -> Result<()> {
        // set inner
        self.global_state.set_inner(GlobalState {
            owner: self.payer.key(),
            tokens_to_sell,
            total_tokens_to_mint,
            virtual_sol_liquidity,
            virtual_token_liquidity,
            bump: bumps.global_state,
        });
        
        Ok(())
    }
}

