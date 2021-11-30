use anchor_lang::prelude::*;
use anchor_lang::AccountsClose;

declare_id!("govHvVVCZsdJLynaFJdqEWBU9AbJ4aHYdZsWno114V9");

#[program]
pub mod permissionless_verifiable_schema_registry {
    use super::*;

    pub fn init(ctx: Context<Init>, ix: InitIx) -> ProgramResult {
        let registry_context = &mut ctx.accounts.registry_context;
        registry_context.authority = *ctx.accounts.authority.key;
        registry_context.entry_seed = ix.entry_seed;
        registry_context.schema_version = 0;
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
        entry.data = ix.data;
        entry.schema_version = ix.schema_version;
        entry.creator = *ctx.accounts.creator.key;
        entry.created_at = Clock::get().unwrap().unix_timestamp;
        entry.is_verified = false; // default to false
        Ok(())
    }

    pub fn remove_entry(ctx: Context<RemoveEntry>) -> ProgramResult {
        ctx.accounts.entry.close(ctx.accounts.authority.to_account_info()).unwrap();
        Ok(())
    }

    pub fn verify_entry(ctx: Context<VerifyEntry>) -> ProgramResult {
        let entry = &mut ctx.accounts.entry;
        entry.is_verified = true;
        entry.verified_at = Clock::get().unwrap().unix_timestamp;
        Ok(())
    }

    pub fn unverify_entry(ctx: Context<UnverifyEntry>) -> ProgramResult {
        let entry = &mut ctx.accounts.entry;
        entry.is_verified = false;
        Ok(())
    }
    
    pub fn add_schema(ctx: Context<AddSchema>, ix: AddSchemaIx) -> ProgramResult {
        let schema_version = ctx.accounts.registry_context.schema_version + 1;
        let schema = &mut ctx.accounts.schema;
        schema.data = ix.data;
        schema.created_at = Clock::get().unwrap().unix_timestamp;
        schema.version = schema_version;
        ctx.accounts.registry_context.schema_version = schema_version;
        Ok(())
    }

    pub fn remove_schema(ctx: Context<RemoveSchema>) -> ProgramResult {
        ctx.accounts.schema.close(ctx.accounts.authority.to_account_info()).unwrap();
        Ok(())
    }
    
}

///////////////// Instructions /////////////////

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitIx {
    pub bump: u8,
    pub entry_seed: String,
    pub permissionless_add: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AddEntryIx {
    pub bump: u8,
    pub address: Pubkey,
    pub schema_version: u8,
    pub data: String,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AddSchemaIx {
    pub bump: u8,
    pub data: String,
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
        space = 1024,
        seeds = [registry_context.entry_seed.as_ref(), ix.address.as_ref()],
        bump = ix.bump,
    )]
    pub entry: Account<'info, EntryData>,
    #[account(constraint = registry_context.permissionless_add || (registry_context.authority == *creator.to_account_info().key) @ ErrorCode::InsufficientAuthority)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VerifyEntry<'info> {
    // TODO constraint this is the singleton registry context address owned by this instance? Maybe we can rely on account discriminator + owner check
    #[account(mut)]
    pub registry_context: Account<'info, RegistryContext>,
    #[account(mut)]
    pub entry: Account<'info, EntryData>,
    #[account(constraint = registry_context.authority == *authority.to_account_info().key @ ErrorCode::InsufficientAuthority)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UnverifyEntry<'info> {
    // TODO constraint this is the singleton registry context address owned by this instance? Maybe we can rely on account discriminator + owner check
    #[account(mut)]
    pub registry_context: Account<'info, RegistryContext>,
    #[account(mut)]
    pub entry: Account<'info, EntryData>,
    #[account(constraint = registry_context.authority == *authority.to_account_info().key @ ErrorCode::InsufficientAuthority)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct RemoveEntry<'info> {
    // TODO constraint this is the singleton registry context address owned by this instance? Maybe we can rely on account discriminator + owner check
    #[account(mut)]
    pub registry_context: Account<'info, RegistryContext>,
    #[account(mut)]
    pub entry: Account<'info, EntryData>,
    #[account(constraint = registry_context.authority == *authority.to_account_info().key || entry.creator == *authority.to_account_info().key @ ErrorCode::InsufficientAuthority)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(ix: AddSchemaIx)]
pub struct AddSchema<'info> {
    #[account(mut)]
    pub registry_context: Account<'info, RegistryContext>,
    #[account(
        init,
        payer = creator,
        // extra space for future upgrades
        space = 1024,
        seeds = [b"schema".as_ref(), &[registry_context.schema_version + 1]],
        bump = ix.bump,
    )]
    pub schema: Account<'info, Schema>,
    #[account(constraint = registry_context.authority == *creator.to_account_info().key @ ErrorCode::InsufficientAuthority)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RemoveSchema<'info> {
    #[account(mut)]
    pub registry_context: Account<'info, RegistryContext>,
    #[account(mut)]
    pub schema: Account<'info, Schema>,
    #[account(constraint = registry_context.authority == *authority.to_account_info().key @ ErrorCode::InsufficientAuthority)]
    pub authority: Signer<'info>,
}

///////////////// DATA /////////////////

#[account]
#[derive(Default)]
pub struct RegistryContext{
    pub authority: Pubkey,
    pub entry_seed: String,
    pub permissionless_add: bool,
    pub schema_version: u8,
}

#[account]
pub struct EntryData {
    pub address: Pubkey,
    pub creator: Pubkey,
    pub created_at: i64,
    pub is_verified: bool,
    pub verified_at: i64,
    pub schema_version: u8,
    pub data: String,
}

#[account]
pub struct Schema {
    pub created_at: i64,
    pub version: u8,
    pub data: String,
}

///////////////// ERRORS /////////////////

#[error]
pub enum ErrorCode {
    #[msg("User does not have authority to modify this registry entry")]
    InsufficientAuthority,
}