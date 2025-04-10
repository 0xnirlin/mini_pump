use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenInterface, TokenAccount, mint_to, MintTo},
    metadata::{
        create_metadata_accounts_v3,
        mpl_token_metadata::types::DataV2,
        CreateMetadataAccountsV3, 
        Metadata as Metaplex,
        mpl_token_metadata::ID as METAPLEX_ID,
    },
};

use crate::state::global_state::GlobalState;
use crate::state::bonding_curve::BondingCurve;

/// # LaunchCoin Instruction
///
/// This instruction initializes a new token with a bonding curve mechanism for price discovery.
/// It creates a new SPL token, sets up its metadata, mints the initial supply, and configures
/// the bonding curve parameters that will govern the token's price dynamics.
///
/// ## Bonding Curve Mechanism
/// The bonding curve implements a constant product formula (similar to AMMs) where:
/// - Price increases as more tokens are purchased
/// - Price decreases as tokens are sold back
/// - Virtual liquidity parameters control the initial price and curve steepness
#[derive(Accounts)]
pub struct LaunchCoin<'info> {
    /// The account paying for the initialization costs
    /// This account must be a signer and will pay for all account creation fees
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Global state account containing protocol-wide parameters
    /// Provides the initial virtual liquidity values for the bonding curve
    #[account(mut)]
    pub global_state: Account<'info, GlobalState>,

    /// The bonding curve account that will be initialized
    /// This PDA is derived from "bonding_curve" and the token mint address
    /// Stores the parameters that control the token's price dynamics
    #[account(init,
    payer = payer,
    space = 8 + BondingCurve::INIT_SPACE,
    seeds = ["bonding_curve".as_bytes(), token_mint.key().as_ref()],
    bump,
    )]
    pub bonding_curve: Account<'info, BondingCurve>,

    /// SOL escrow account that will hold SOL from token purchases
    /// This PDA is derived from "bonding_curve_sol_escrow" and the bonding curve address
    #[account(
        seeds = ["bonding_curve_sol_escrow".as_bytes(), bonding_curve.key().as_ref()],
        bump,
    )]
    pub bonding_curve_sol_escrow: SystemAccount<'info>,

    /// The token mint that will be created
    /// The bonding curve will be the mint authority and freeze authority
    #[account(
        init,
        payer = payer,
        mint::decimals = 6,
        mint::authority = bonding_curve,
        mint::freeze_authority = bonding_curve,
    )]
    pub token_mint: InterfaceAccount<'info, Mint>,

    /// Token account owned by the bonding curve
    /// Will hold the initial token supply that will be sold through the bonding curve
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = token_mint,
        associated_token::authority = bonding_curve,
    )]
    pub bonding_curve_token_account: InterfaceAccount<'info, TokenAccount>,

    /// SPL Token program for token operations
    pub token_program: Interface<'info, TokenInterface>,

    /// Metaplex Token Metadata program for creating token metadata
    /// This program is required for creating and managing the token's metadata
    /// including name, symbol, and URI for off-chain assets
    #[account(address = METAPLEX_ID)]
    pub token_metadata_program: Program<'info, Metaplex>,

    /// Associated Token program for creating token accounts
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// System program for creating accounts
    pub system_program: Program<'info, System>,

    /// Rent sysvar for rent exemption calculations
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> LaunchCoin<'info> {
    /// Launches a new token with a bonding curve mechanism
    ///
    /// This function performs the complete token initialization process:
    /// 1. Creates token metadata with the provided name, symbol, and URI
    /// 2. Mints the initial token supply to the bonding curve's token account
    /// 3. Initializes the bonding curve with virtual liquidity parameters
    /// 4. Emits a launch event with key token information
    ///
    /// ## Parameters
    /// - `name`: The name of the token (e.g., "Mini Pump Token")
    /// - `symbol`: The token symbol (e.g., "MPT")
    /// - `uri`: URL to the token's metadata JSON
    /// - `bumps`: Bump seeds for PDAs used in the instruction
    ///
    /// ## Returns
    /// - `Result<()>`: Success or error
    pub fn launch_coin(&mut self, name: String, symbol: String, uri: String, bumps: LaunchCoinBumps) -> Result<()> {
        // Create the token metadata structure with the provided information
        let token_data = DataV2 {
            name,
            symbol,
            uri,
            seller_fee_basis_points: 0,  // No royalty fees
            creators: None,               // No creators specified
            collection: None,             // Not part of a collection
            uses: None,                   // No uses metadata
        };

        // Prepare the PDA signer seeds for the bonding curve
        // This allows the bonding curve PDA to sign for metadata creation
        let token_mint_key = self.token_mint.to_account_info().key();
        let seeds = &[
            b"bonding_curve".as_ref(),
            token_mint_key.as_ref(),
            &[bumps.bonding_curve],
        ];

        let signer = &[&seeds[..]];

        // Create the token metadata using the Metaplex program
        // This sets up the token's name, symbol, and URI that will be visible in wallets
        let metadata_ctx = CpiContext::new_with_signer(
            self.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: self.token_mint.to_account_info(),
                mint: self.token_mint.to_account_info(),
                mint_authority: self.bonding_curve.to_account_info(),
                update_authority: self.bonding_curve.to_account_info(),
                payer: self.payer.to_account_info(),
                system_program: self.system_program.to_account_info(),
                rent: self.rent.to_account_info(),
            },
            signer,
        );

        // Execute the metadata creation with parameters:
        // - token_data: The token metadata
        // - is_mutable: false (metadata cannot be changed)
        // - update_authority_is_signer: true (update authority is signing)
        // - collection_details: None (not part of a collection)
        create_metadata_accounts_v3(metadata_ctx, token_data, false, true, None)?;

        // Mint the initial token supply to the bonding curve's token account
        // This creates 1 billion tokens (with 6 decimals) that will be sold through the bonding curve
        mint_to(CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            MintTo {
                mint: self.token_mint.to_account_info(),
                to: self.bonding_curve_token_account.to_account_info(),
                authority: self.bonding_curve.to_account_info(),
            },
            &[&[
                b"bonding_curve",
                self.token_mint.key().as_ref(),
                &[bumps.bonding_curve],
            ]],
        ), 1_000_000_000_000_000)?; // 1 billion tokens with 6 decimals
        
        msg!("Launching coin");
        
        // Initialize the bonding curve with parameters from the global state
        // This sets up the virtual liquidity values that determine the token's price curve
        self.bonding_curve.set_inner(BondingCurve {
            // Initial virtual SOL liquidity (affects starting price)
            virtual_sol_liquidity: self.global_state.virtual_sol_liquidity,
            // Initial virtual token liquidity (affects curve steepness)
            virtual_token_liquidity: self.global_state.virtual_token_liquidity,
            // No tokens sold initially
            tokens_sold: 0,
            // Reference to the token mint
            token_mint: self.token_mint.key(),
            // Bonding curve is active and ready for trading
            is_active: true,
            // Store the bump for future PDA derivation
            bump: bumps.bonding_curve,
        });

        // Emit an event to notify listeners about the token launch
        self.emit_launch_event();

        Ok(())
    }
    
}

/// Event emitted when a new token is launched
/// 
/// This event provides key information about the newly created token and its bonding curve,
/// allowing off-chain services to track token launches and their parameters.
#[event]
pub struct LaunchTokens {
    /// The address of the token mint
    pub token_mint: Pubkey,
    /// The address of the bonding curve account
    pub bonding_curve: Pubkey,
    /// Initial virtual SOL liquidity (affects starting price)
    pub virtual_sol_liquidity: u64,
    /// Initial virtual token liquidity (affects curve steepness)
    pub virtual_token_liquidity: u64,
    /// Total number of tokens minted initially
    pub total_tokens_minted: u64,
    /// Unix timestamp of the launch
    pub timestamp: i64,
}

impl<'info> LaunchCoin<'info> {
    /// Emits the LaunchTokens event with information about the newly launched token
    ///
    /// This function creates and emits an event containing key information about the token launch,
    /// including the token mint address, bonding curve parameters, and timestamp.
    pub fn emit_launch_event(&self) {
        emit!(LaunchTokens {
            // Address of the token mint for tracking
            token_mint: self.token_mint.key(),
            // Address of the bonding curve for reference
            bonding_curve: self.bonding_curve.key(),
            // Initial virtual SOL liquidity from global state
            virtual_sol_liquidity: self.global_state.virtual_sol_liquidity,
            // Initial virtual token liquidity from global state
            virtual_token_liquidity: self.global_state.virtual_token_liquidity,
            // Total tokens minted (1 billion with 6 decimals)
            total_tokens_minted: 1_000_000_000_000_000, // Same as the amount minted
            // Current blockchain timestamp
            timestamp: Clock::get().unwrap().unix_timestamp,
        });
    }
}
