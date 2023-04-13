use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{create_metadata_accounts_v3, CreateMetadataAccountsV3, Metadata},
    token::{mint_to, transfer, Mint, MintTo, Token, TokenAccount, Transfer},
};
use mpl_token_metadata::{pda::find_metadata_account, state::DataV2};

declare_id!("7gZTdZg86YsYbs92Rhv63kZUAkoww1kLexJg8sNpgVQ3");

#[program]
pub mod anchor_token {
    use super::*;
    // Create new token mint with PDA as mint authority
    pub fn create_mint(
        ctx: Context<CreateMint>,
        uri: String,
        name: String,
        symbol: String,
    ) -> Result<()> {
        // create metadata account for token mint
        let signer_seeds: &[&[&[u8]]] =
            &[&[b"reward", &[*ctx.bumps.get("reward_token_mint").unwrap()]]];
        create_metadata_accounts_v3(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    metadata: ctx.accounts.metadata_account.to_account_info(), // the metadata account being created
                    mint: ctx.accounts.reward_token_mint.to_account_info(), // the mint account of the metadata account
                    mint_authority: ctx.accounts.reward_token_mint.to_account_info(), // the mint authority of the mint account
                    update_authority: ctx.accounts.reward_token_mint.to_account_info(), // the update authority of the metadata account
                    payer: ctx.accounts.signer.to_account_info(), // the payer for creating the metadata account
                    system_program: ctx.accounts.system_program.to_account_info(), // the system program account
                    rent: ctx.accounts.rent.to_account_info(), // the rent sysvar account
                },
                signer_seeds,
            ),
            DataV2 {
                name: name,     // on-chain name
                symbol: symbol, // on-chain symbol
                uri: uri,       // off-chain metadata
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            },
            true, // is_mutable
            true, // update_authority_is_signer
            None, // collection details
        )?;
        Ok(())
    }

    // Mint tokens to player token account
    pub fn mint_tokens(ctx: Context<MintTokens>, amount: u64) -> Result<()> {
        let signer_seeds: &[&[&[u8]]] =
            &[&[b"reward", &[*ctx.bumps.get("reward_token_mint").unwrap()]]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.reward_token_mint.to_account_info(),
                to: ctx.accounts.player_token_account.to_account_info(),
                authority: ctx.accounts.reward_token_mint.to_account_info(),
            },
            signer_seeds,
        );

        mint_to(cpi_ctx, amount)?;
        Ok(())
    }

    // Deposit tokens from player token account to vault token account
    pub fn deposit_tokens(ctx: Context<DepositTokens>, amount: u64) -> Result<()> {
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.player_token_account.to_account_info(),
                to: ctx.accounts.vault_token_account.to_account_info(),
                authority: ctx.accounts.player.to_account_info(),
            },
        );

        transfer(cpi_ctx, amount)?;
        Ok(())
    }

    // Withdraw tokens from vault token account to player token account
    pub fn withdraw_tokens(ctx: Context<WithdrawTokens>, amount: u64) -> Result<()> {
        let reward_token_mint = ctx.accounts.reward_token_mint.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"vault",
            reward_token_mint.as_ref(),
            &[*ctx.bumps.get("vault_token_account").unwrap()],
        ]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_token_account.to_account_info(),
                to: ctx.accounts.player_token_account.to_account_info(),
                authority: ctx.accounts.vault_token_account.to_account_info(),
            },
            signer_seeds,
        );

        transfer(cpi_ctx, amount)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateMint<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    // The PDA is both the address of the mint account and the mint authority
    #[account(
        init,
        seeds = [b"reward"],
        bump,
        payer = signer,
        mint::decimals = 9,
        mint::authority = reward_token_mint,

    )]
    pub reward_token_mint: Account<'info, Mint>,

    ///CHECK:
    #[account(
        mut,
        address=find_metadata_account(&reward_token_mint.key()).0
    )]
    pub metadata_account: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct MintTokens<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    // Initialize player token account if it doesn't exist
    #[account(
        init_if_needed,
        payer = player,
        associated_token::mint = reward_token_mint,
        associated_token::authority = player
    )]
    pub player_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"reward"],
        bump,
    )]
    pub reward_token_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositTokens<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    // Player token account
    #[account(
        mut,
        associated_token::mint = reward_token_mint,
        associated_token::authority = player
    )]
    pub player_token_account: Account<'info, TokenAccount>,

    // Initialize vault token account if it doesn't exist
    // The PDA is both the address of the token account and the token account authority
    #[account(
        init_if_needed,
        seeds = [b"vault", reward_token_mint.key().as_ref()],
        bump,
        payer = player,
        token::mint=reward_token_mint,
        token::authority=vault_token_account,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"reward"],
        bump,
    )]
    pub reward_token_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawTokens<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    // Initialize player token account if it doesn't exist
    #[account(
        init_if_needed,
        payer = player,
        associated_token::mint = reward_token_mint,
        associated_token::authority = player
    )]
    pub player_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"vault", reward_token_mint.key().as_ref()],
        bump,
        token::mint=reward_token_mint,
        token::authority=vault_token_account,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"reward"],
        bump,
    )]
    pub reward_token_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
