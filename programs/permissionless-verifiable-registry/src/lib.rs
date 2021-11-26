use anchor_lang::prelude::*;
use anchor_lang::AccountsClose;

declare_id!("govHvVVCZsdJLynaFJdqEWBU9AbJ4aHYdZsWno114V9");

#[program]
pub mod permissionless_verifiable_registry {
    use super::*;

    pub fn init(ctx: Context<Init>, ix: InitIx) -> ProgramResult {
        let registry_context = &mut ctx.accounts.registry_context;
        registry_context.authority = *ctx.accounts.authority.key;
        registry_context.entry_seed = ix.entry_seed;
        Ok(())
    }

    pub fn transfer_authority(ctx: Context<TransferAuthority>) -> ProgramResult {
        let registry_context = &mut ctx.accounts.registry_context;
        registry_context.authority = *ctx.accounts.new_authority.key;
        Ok(())
    }
    
    pub fn add_entry(ctx: Context<AddEntry>, ix: AddEntryIx) -> ProgramResult {
        let entry = &mut ctx.accounts.entry;
        entry.address = ix.address;
        entry.data_url = ix.data_url;
        entry.creator = *ctx.accounts.creator.key;
        entry.created_at = Clock::get().unwrap().unix_timestamp;
        entry.is_verified = false; // default to false
        Ok(())
    }

    pub fn verify_entry(ctx: Context<VerifyEntry>) -> ProgramResult {
        let entry = &mut ctx.accounts.entry;
        entry.is_verified = true;
        entry.verified_at = Clock::get().unwrap().unix_timestamp;
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
    pub entry_seed: String,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AddEntryIx {
    pub bump: u8,
    pub data_url: String,
    pub address: Pubkey,
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
    pub registry_context: Account<'info, RegistryContext>,
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TransferAuthority<'info> {
    // TODO constraint this is the singleton registry context address? Maybe we can rely on account discriminator + owner check
    #[account(mut)]
    pub registry_context: Account<'info, RegistryContext>,
    #[account(constraint = registry_context.authority == *authority.to_account_info().key @ ErrorCode::InsufficientAuthority)]
    pub authority: Signer<'info>,
    pub new_authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(ix: AddEntryIx)]
pub struct AddEntry<'info> {
    // TODO constraint this is the singleton registry context address owned by this instance? Maybe we can rely on account discriminator + owner check
    #[account(mut)]
    pub registry_context: Account<'info, RegistryContext>,
    #[account(
        init,
        payer = creator,
        // extra space for future upgrades
        space = 256,
        seeds = [registry_context.entry_seed.as_ref(), ix.address.as_ref()],
        bump = ix.bump,
    )]
    pub entry: Account<'info, EntryData>,
    // permissionless authority to add entries
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VerifyEntry<'info> {
    #[account(mut)]
    pub entry: Account<'info, EntryData>,
    // TODO constraint this is the singleton registry context address owned by this instance? Maybe we can rely on account discriminator + owner check
    #[account(mut)]
    pub registry_context: Account<'info, RegistryContext>,
    #[account(constraint = registry_context.authority == *authority.to_account_info().key @ ErrorCode::InsufficientAuthority)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct RemoveEntry<'info> {
    #[account(mut)]
    pub entry: Account<'info, EntryData>,
    // TODO constraint this is the singleton registry context address owned by this instance? Maybe we can rely on account discriminator + owner check
    #[account(mut)]
    pub registry_context: Account<'info, RegistryContext>,
    #[account(constraint = registry_context.authority == *authority.to_account_info().key || entry.creator == *authority.to_account_info().key @ ErrorCode::InsufficientAuthority)]
    pub authority: Signer<'info>,
}

///////////////// DATA /////////////////

#[account]
#[derive(Default)]
pub struct RegistryContext{
    pub authority: Pubkey,
    pub entry_seed: String,
}

#[account]
#[derive(Default)]
pub struct EntryData {
    pub address: Pubkey,
    // only data here is just a URL. JSON must contain a schema version
    pub data_url: String,
    pub creator: Pubkey,
    pub is_verified: bool,
    pub verified_at: i64,
    pub created_at: i64,
}

///////////////// ERRORS /////////////////

#[error]
pub enum ErrorCode {
    #[msg("User does not have authority to modify this registry entry")]
    InsufficientAuthority,
}