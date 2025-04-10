/*
 ███╗   ███╗██╗███╗   ██╗██╗    ██████╗ ██╗   ██╗███╗   ███╗██████╗ 
 ████╗ ████║██║████╗  ██║██║    ██╔══██╗██║   ██║████╗ ████║██╔══██╗
 ██╔████╔██║██║██╔██╗ ██║██║    ██████╔╝██║   ██║██╔████╔██║██████╔╝
 ██║╚██╔╝██║██║██║╚██╗██║██║    ██╔═══╝ ██║   ██║██║╚██╔╝██║██╔═══╝ 
 ██║ ╚═╝ ██║██║██║ ╚████║██║    ██║     ╚██████╔╝██║ ╚═╝ ██║██║     
 ╚═╝     ╚═╝╚═╝╚═╝  ╚═══╝╚═╝    ╚═╝      ╚═════╝ ╚═╝     ╚═╝╚═╝     
                                                                     
  ✨ Bonding Curve Token Launch Protocol for Solana ✨

 💹 Price
   ^
   |                                 /|
   |                               /
   |                             /
   |                           /
   |                        /
   |                     /
   |                  /
   |              /
   |         _/
   |___----
   +---------------------------------> Tokens Sold
 
 🔹 Fair Price Discovery - Transparent token pricing based on supply and demand
 🔹 Automatic Liquidity - Self-sustaining liquidity mechanism with no impermanent loss
 🔹 Seamless DEX Migration - Smooth transition to decentralized exchanges
 🔹 Zero Pre-mine - Fair distribution with no team allocation
 🔹 Constant Product Formula - Mathematically sound price discovery
 
 🚀 Launch → 📈 Trade → 🔄 Migrate → 💰 Profit

 v1.0.0 | github.com/0xnirlin/mini-pump 
*/

use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("GgumMKBeidaDAeMFHxP4ejUsoHBkMYnihxLCzVzpNJzv");

#[program]
pub mod mini_pump {
    use super::*;

    pub fn init_protocol(ctx: Context<InitProtocol>, total_tokens_to_mint: u64, virtual_sol_liquidity: u64, virtual_token_liquidity: u64, tokens_to_sell: Pubkey) -> Result<()> {
        ctx.accounts.init_protocol(total_tokens_to_mint, virtual_sol_liquidity, virtual_token_liquidity, tokens_to_sell, ctx.bumps)
    }

    pub fn launch_coin(ctx: Context<LaunchCoin>, name: String, symbol: String, uri: String) -> Result<()> {
        ctx.accounts.launch_coin( name, symbol, uri, ctx.bumps)
    }

    pub fn buy_token(ctx: Context<TradeCoin>, sol_amount: u64) -> Result<()> {
        ctx.accounts.buy_token(sol_amount)
    }

    pub fn sell_token(ctx: Context<TradeCoin>, token_amount: u64) -> Result<()> {
        ctx.accounts.sell_token(token_amount)
    }

    pub fn withdraw_funds(ctx: Context<WithdrawFunds>) -> Result<()> {
        ctx.accounts.withdraw_funds()
    }
}
