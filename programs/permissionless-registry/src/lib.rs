use anchor_lang::prelude::*;
use anchor_lang::AccountsClose;

declare_id!("govHvVVCZsdJLynaFJdqEWBU9AbJ4aHYdZsWno114V9");

#[program]
pub mod permissionlessregistry {
    use super::*;

    pub fn add_entry(ctx: Context<AddEntry>, ix: AddEntryIx) -> ProgramResult {
        let clock = Clock::get().unwrap();
        let timestamp = clock.unix_timestamp;

        let mut entry_data = ix.entry;
        let entry_account = &mut ctx.accounts.entry;
        entry_data.authority = *ctx.accounts.user.key;
        entry_account.data = entry_data;
        entry_account.created_at = timestamp;
        entry_account.updated_at = timestamp;
        Ok(())
    }

    pub fn update_entry(ctx: Context<UpdateEntry>, entry_data: EntryData) -> ProgramResult {
        let clock = Clock::get().unwrap();
        let timestamp = clock.unix_timestamp;

        let entry_account = &mut ctx.accounts.entry;
        entry_account.data = entry_data;
        entry_account.updated_at = timestamp;
        Ok(())
    }

    pub fn remove_entry(ctx: Context<RemoveEntry>) -> ProgramResult {
        ctx.accounts.entry.close(ctx.accounts.user.to_account_info()).unwrap();
        Ok(())
    }
}

///////////////// Instructions /////////////////

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AddEntryIx {
    pub entry: EntryData,
    pub bump: u8,
    pub seed: [u8; 32],
}

///////////////// Contexts /////////////////

#[derive(Accounts)]
#[instruction(ix: AddEntryIx)]
pub struct AddEntry<'info> {
    #[account(
        init,
        payer = user,
        space = 1024,
        seeds = [b"governance-program".as_ref(), ix.seed.as_ref()],
        bump = ix.bump,
    )]
    pub entry: Account<'info, GovernanceProgramAccount>,
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateEntry<'info> {
    pub entry: Account<'info, GovernanceProgramAccount>,
    #[account(constraint = entry.data.authority == *user.key @ ErrorCode::InsufficientAuthority)]
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct RemoveEntry<'info> {
    #[account(mut)]
    pub entry: Account<'info, GovernanceProgramAccount>,
    #[account(constraint = entry.data.authority == *user.key @ ErrorCode::InsufficientAuthority)]
    pub user: Signer<'info>,
}

///////////////// DATA /////////////////

#[derive(Default, Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct EntryData {
    pub name: String,
    pub description: String,
    pub image_url: String,
    pub program_address: Pubkey,
    pub additional_data_url: String,
    pub authority: Pubkey,
}

#[account]
#[derive(Default)]
pub struct GovernanceProgramAccount {
    pub data: EntryData,
    pub created_at: i64,
    pub updated_at: i64,
}

#[error]
pub enum ErrorCode {
    #[msg("User does not have authority to modify this registry")]
    InsufficientAuthority,
}