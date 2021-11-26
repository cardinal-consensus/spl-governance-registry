use anchor_lang::prelude::*;
use anchor_lang::AccountsClose;

declare_id!("govHvVVCZsdJLynaFJdqEWBU9AbJ4aHYdZsWno114V9");

#[program]
pub mod verified_registry {
    use super::*;

    pub fn init(ctx: Context<Init>, _ix: InitIx) -> ProgramResult {
        let registry_context = &mut ctx.accounts.registry_context;
        registry_context.authority = *ctx.accounts.authority.key;
        Ok(())
    }

    pub fn transfer_authority(ctx: Context<TransferVerificationAuthority>) -> ProgramResult {
        let registry_context = &mut ctx.accounts.registry_context;
        registry_context.authority = *ctx.accounts.new_authority.key;
        Ok(())
    }
    
    pub fn add_entry(ctx: Context<AddEntry>, ix: AddEntryIx) -> ProgramResult {
        let clock = Clock::get().unwrap();
        let timestamp = clock.unix_timestamp;
        let entry = &mut ctx.accounts.entry;
        entry.address = ix.address;
        entry.additional_data_url = ix.additional_data_url;
        entry.created_at = timestamp;
        entry.updated_at = timestamp;
        Ok(())
    }

    pub fn update_entry(ctx: Context<UpdateEntry>, ix: UpdateEntryIx) -> ProgramResult {
        let clock = Clock::get().unwrap();
        let timestamp = clock.unix_timestamp;
        let entry = &mut ctx.accounts.entry;
        entry.additional_data_url = ix.additional_data_url;
        entry.updated_at = timestamp;
        Ok(())
    }

    pub fn remove_entry(ctx: Context<RemoveEntry>) -> ProgramResult {
        ctx.accounts.entry.close(ctx.accounts.authority.to_account_info()).unwrap();
        Ok(())
    }
}

///////////////// Instructions /////////////////

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitIx {
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AddEntryIx {
    pub address: Pubkey,
    pub additional_data_url: String,
    pub bump: u8,
    pub seed: [u8; 32],
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UpdateEntryIx {
    pub additional_data_url: String,
}

///////////////// Contexts /////////////////
#[derive(Accounts)]
#[instruction(ix: InitIx)]
pub struct Init<'info> {
    #[account(
        init,
        payer = authority,
        // extra space for future upgrades
        space = 128,
        seeds = [b"registry-context".as_ref()],
        bump = ix.bump,
    )]
    pub registry_context: Account<'info, RegistryContextAccount>,
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TransferVerificationAuthority<'info> {
    // TODO constraint this is the singleton registry context address? Maybe we can rely on account discriminator + owner check
    #[account(mut)]
    pub registry_context: Account<'info, RegistryContextAccount>,
    #[account(constraint = registry_context.authority == *authority.to_account_info().key @ ErrorCode::InsufficientAuthority)]
    pub authority: Signer<'info>,
    pub new_authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(ix: AddEntryIx)]
pub struct AddEntry<'info> {
    #[account(mut)]
    pub registry_context: Account<'info, RegistryContextAccount>,
    #[account(
        init,
        payer = authority,
        // extra space for future upgrades
        space = 256,
        // TODO constraint seed == ix.address? or just use address as seed?
        seeds = [b"governance-program".as_ref(), ix.seed.as_ref()],
        bump = ix.bump,
    )]
    pub entry: Account<'info, EntryData>,
    #[account(constraint = registry_context.authority == *authority.to_account_info().key @ ErrorCode::InsufficientAuthority)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateEntry<'info> {
    #[account(mut)]
    pub registry_context: Account<'info, RegistryContextAccount>,
    #[account(mut)]
    pub entry: Account<'info, EntryData>,
    #[account(constraint = registry_context.authority == *authority.to_account_info().key @ ErrorCode::InsufficientAuthority)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct RemoveEntry<'info> {
    // TODO constraint this is the singleton registry context address? Maybe we can rely on account discriminator + owner check
    #[account(mut)]
    pub registry_context: Account<'info, RegistryContextAccount>,
    #[account(mut)]
    pub entry: Account<'info, EntryData>,
    #[account(constraint = registry_context.authority == *authority.to_account_info().key @ ErrorCode::InsufficientAuthority)]
    pub authority: Signer<'info>,
}

///////////////// DATA /////////////////

#[account]
#[derive(Default)]
pub struct RegistryContextAccount{
    pub authority: Pubkey,
}

#[account]
#[derive(Default)]
pub struct EntryData {
    pub address: Pubkey,
    pub additional_data_url: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[error]
pub enum ErrorCode {
    #[msg("User does not have authority to modify this registry")]
    InsufficientAuthority,
}