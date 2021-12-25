use anchor_lang::prelude::*;

declare_id!("2hCEoMSLuA7ShgWcWHqfNoic4FmN2NvKp7T8TEwbmomw");

use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Burn, Mint, MintTo, SetAuthority, Token, TokenAccount, Transfer},
};
use spl_token::instruction::AuthorityType;

#[program]
pub mod scallop {
    use super::*;

    const SCALLOP_COUPON_SEED:&[u8] = b"scallop_coupon";
    const SCALLOP_VAULT_SEED: &[u8] = b"scallop_vault";

    pub fn initialize(
        ctx: Context<Initialize>,
        _vault_account_bump: u8,
        coupon_price: u64,
    ) -> ProgramResult {
        // let (coupon_authority, _coupon_authority_bump) =
        //     Pubkey::find_program_address(&[SCALLOP_COUPON_SEED], ctx.program_id);

        let (vault_authority, vault_authority_bump) =
            Pubkey::find_program_address(&[SCALLOP_VAULT_SEED], ctx.program_id);
        token::set_authority(
            ctx.accounts.into_set_authority_context(),
            AuthorityType::AccountOwner,
            Some(vault_authority),
        )?;

        let scallop_tank = &mut ctx.accounts.scallop_tank;
        scallop_tank.coupon_price = coupon_price;
        // scallop_tank.coupon_authority = coupon_authority;
        scallop_tank.coupon_mint = *ctx.accounts.coupon_mint.to_account_info().key;
        scallop_tank.vault = *ctx.accounts.scallop_vault.to_account_info().key;
        scallop_tank.vault_authority = vault_authority;
        scallop_tank.vault_authority_bump = vault_authority_bump;
        
        Ok(())
    }

    /// Deposit tokens into a reserve
    pub fn deposit(ctx: Context<Deposit>, number_of_coupon: u64) -> ProgramResult {
        let (_coupon_authority, coupon_authority_bump) =
            Pubkey::find_program_address(&[SCALLOP_COUPON_SEED], ctx.program_id);
        let authority_seed = &[&SCALLOP_COUPON_SEED[..],&[coupon_authority_bump]];

        token::transfer(
            ctx.accounts.into_transfer_context(),
            number_of_coupon * ctx.accounts.scallop_tank.coupon_price,
        )?;

        token::mint_to(
            ctx.accounts
                .into_mint_context()
                .with_signer(&[&authority_seed[..]]),
            number_of_coupon * ctx.accounts.scallop_tank.coupon_price,
        )?;

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, number_of_coupon: u64) -> ProgramResult {

        token::transfer(
            ctx.accounts
                .into_transfer_context()
                .with_signer(&[&[SCALLOP_VAULT_SEED, &[ctx.accounts.scallop_tank.vault_authority_bump]]]),
            number_of_coupon * ctx.accounts.scallop_tank.coupon_price,
        )?;
        
        token::burn(
            ctx.accounts.into_burn_context(),
            number_of_coupon * ctx.accounts.scallop_tank.coupon_price,
        )?;

        Ok(())
    }
}

#[account]
pub struct ScallopTank {
    pub coupon_price: u64,
    // pub coupon_authority: Pubkey,
    pub coupon_mint: Pubkey,
    pub vault: Pubkey,
    pub vault_authority: Pubkey,
    pub vault_authority_bump: u8,
}

#[derive(Accounts)]
#[instruction(mint_bump: u8, vault_bump: u8, coupon_price: u64)]
pub struct Initialize<'info> {
    pub initializer: Signer<'info>,

    #[account(init, payer = initializer, space = 8 + 40)]
    pub scallop_tank: Account<'info, ScallopTank>,

    #[account(
        init, payer = initializer,
        seeds = [b"scallop_coupon"], bump = mint_bump,
        mint::decimals = 0, mint::authority = coupon_mint,
    )]
    pub coupon_mint: Account<'info, Mint>,

    #[account(
        init, payer = initializer,
        seeds = [b"scallop_vault".as_ref()], bump = vault_bump,
        token::mint = mint, token::authority = initializer,
    )]
    pub scallop_vault: Account<'info, TokenAccount>,

    /// The mint for the token to be deposited
    pub mint: Account<'info, Mint>,
    
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct Deposit<'info> {
    pub depositor: Signer<'info>,    

    pub scallop_tank: Account<'info, ScallopTank>,

    /// The account that will store the lottery coupons
    #[account(
        init_if_needed,
        payer = depositor,
        associated_token::mint = coupon_mint,
        associated_token::authority = depositor,
    )]
    pub depositor_coupon_account: Account<'info, TokenAccount>,

    /// The token account with the tokens to be deposited
    #[account(mut)]
    pub depositor_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub scallop_vault: Account<'info, TokenAccount>,

    // pub coupon_authority: AccountInfo<'info>, 
    // The mint for the deposit notes
    pub coupon_mint: Account<'info, Mint>,

    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    pub depositor: Signer<'info>,    

    pub scallop_tank: Account<'info, ScallopTank>,

    /// The account that stores the lottery coupons
    #[account(
        mut,
        associated_token::mint = coupon_mint,
        associated_token::authority = depositor,
    )]
    pub depositor_coupon_account: Account<'info, TokenAccount>,

    /// The token account with the tokens to be deposited
    #[account(mut)]
    pub depositor_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub scallop_vault: Account<'info, TokenAccount>,

    // The mint for the deposit notes
    pub coupon_mint: Account<'info, Mint>,

    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}


impl<'info> Initialize<'info> {
    fn into_set_authority_context(&self) -> CpiContext<'_,'_,'_,'info, SetAuthority<'info>> {
        CpiContext::new(
            self.token_program.to_account_info().clone(),
            SetAuthority {
                account_or_mint: self.scallop_vault.to_account_info().clone(),
                current_authority: self.initializer.to_account_info().clone(),
            }
        )
    }
}

impl<'info> Deposit<'info> {
    fn into_transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.depositor_token_account.to_account_info().clone(),
                to: self.scallop_vault.to_account_info().clone(),
                authority: self.depositor.to_account_info().clone(),
            },
        )
    }

    fn into_mint_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            MintTo {
                to: self.depositor_coupon_account.to_account_info().clone(),
                mint: self.coupon_mint.to_account_info().clone(),
                authority: self.coupon_mint.to_account_info().clone(),
            }
        )
    }
}

impl<'info> Withdraw<'info> {
    fn into_transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.scallop_vault.to_account_info().clone(),
                to: self.depositor_token_account.to_account_info().clone(),
                authority: self.scallop_tank.to_account_info().clone(),
            },
        )
    }

    fn into_burn_context(&self) -> CpiContext<'_, '_, '_, 'info, Burn<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Burn {
                to: self.depositor_coupon_account.to_account_info().clone(),
                mint: self.coupon_mint.to_account_info().clone(),
                authority: self.depositor.to_account_info().clone(),
            }
        )
    }
}