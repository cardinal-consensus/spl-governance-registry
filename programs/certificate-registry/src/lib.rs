use anchor_lang::prelude::*;
use anchor_lang::AccountsClose;
use anchor_spl::token::{Mint};

declare_id!("certvKPnmaaxL84RuzvLVJcNh2zvdUxqjcsJQCsHEFE");

#[program]
pub mod permissionless_verifiable_registry {
    use super::*;

    pub fn issue(ctx: Context<Issue>, ix: IssueIx) -> ProgramResult {
        let entry = &mut ctx.accounts.entry;
        entry.mint_address = ctx.accounts.mint.key();
        entry.owner = ix.owner;
        entry.data = ix.data;
        entry.schema_version = ix.schema_version;
        entry.issuer = *ctx.accounts.issuer.key;
        entry.issued_at = Clock::get().unwrap().unix_timestamp;
        Ok(())
    }

    pub fn revoke(ctx: Context<Revoke>) -> ProgramResult {
        ctx.accounts.entry.close(ctx.accounts.user.to_account_info()).unwrap();
        Ok(())
    }
}

///////////////// Instructions /////////////////

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct IssueIx {
    pub bump: u8,
    pub owner: Pubkey,
    pub schema_version: u8,
    pub data: String,
}

///////////////// Contexts /////////////////

#[derive(Accounts)]
#[instruction(ix: IssueIx)]
pub struct Issue<'info> {
    #[account(
        init,
        payer = issuer,
        // extra space for future upgrades
        space = 1024,
        seeds = [b"certificate", mint.to_account_info().key.as_ref(), ix.owner.as_ref()],
        bump = ix.bump,
    )]
    pub entry: Account<'info, EntryData>,
    pub mint: Account<'info, Mint>,
    // #[account(constraint = mint.mint_authority == *issuer.to_account_info().key @ ErrorCode::InsufficientAuthority)]
    pub issuer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Revoke<'info> {
    #[account(mut)]
    pub entry: Account<'info, EntryData>,
    #[account(constraint = entry.issuer == *user.to_account_info().key || entry.owner == *user.to_account_info().key @ ErrorCode::InsufficientAuthority)]
    pub user: Signer<'info>,
}

///////////////// DATA /////////////////

#[account]
pub struct EntryData {
    pub mint_address: Pubkey,
    pub owner: Pubkey,
    pub issuer: Pubkey,
    pub issued_at: i64,
    pub schema_version: u8,
    pub data: String,
}

///////////////// ERRORS /////////////////

#[error]
pub enum ErrorCode {
    #[msg("User does not have authority to modify this registry entry")]
    InsufficientAuthority,
}