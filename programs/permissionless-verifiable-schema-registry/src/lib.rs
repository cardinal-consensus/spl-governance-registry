use anchor_lang::prelude::*;
use anchor_lang::AccountsClose;
use anchor_lang::solana_program::{
    program::{invoke},
    system_instruction,
};

declare_id!("tkJqbNU3dk3eCwtT4EjSFisxza8JcuKSbDNbTZDQv76");

#[program]
pub mod permissionless_verifiable_schema_registry {
    use super::*;

    pub fn init(ctx: Context<Init>, ix: InitIx) -> ProgramResult {
        let registry_config = &mut ctx.accounts.registry_config;
        registry_config.authority = *ctx.accounts.authority.key;
        registry_config.entry_seed = ix.entry_seed;
        registry_config.schema_version = 0;
        registry_config.permissionless_add = ix.permissionless_add;
        registry_config.add_fee = ix.add_fee;
        Ok(())
    }

    pub fn transfer_authority(ctx: Context<TransferAuthority>) -> ProgramResult {
        let registry_config = &mut ctx.accounts.registry_config;
        registry_config.authority = *ctx.accounts.new_authority.key;
        Ok(())
    }
    
    pub fn add_entry(ctx: Context<AddEntry>, ix: AddEntryIx) -> ProgramResult {
        let entry = &mut ctx.accounts.entry;
        entry.primary_key = ix.primary_key;
        entry.data = ix.data;
        entry.schema_version = ix.schema_version;
        entry.creator = *ctx.accounts.creator.key;
        entry.created_at = Clock::get().unwrap().unix_timestamp;
        entry.is_verified = false; // default to false

        if ctx.accounts.creator.lamports() < ctx.accounts.registry_config.add_fee {
            return Err(ErrorCode::InsufficientBalance.into());
        }
        invoke(
            &system_instruction::transfer(
                &ctx.accounts.creator.key,
                &ctx.accounts.authority.key,
                ctx.accounts.registry_config.add_fee,
            ),
            &[
                ctx.accounts.creator.to_account_info(),
                ctx.accounts.authority.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

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
        let schema_version = ctx.accounts.registry_config.schema_version + 1;
        let schema = &mut ctx.accounts.schema;
        schema.data = ix.data;
        schema.created_at = Clock::get().unwrap().unix_timestamp;
        schema.version = schema_version;
        ctx.accounts.registry_config.schema_version = schema_version;
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
    pub add_fee: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AddEntryIx {
    pub bump: u8,
    pub primary_key: Vec<u8>,
    pub schema_version: u8,
    pub data: Vec<u8>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AddSchemaIx {
    pub bump: u8,
    pub data: Vec<u8>,
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
        seeds = [b"registry-config".as_ref()],
        bump = ix.bump,
    )]
    pub registry_config: Account<'info, RegistryConfig>,
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TransferAuthority<'info> {
    #[account(mut)]
    pub registry_config: Account<'info, RegistryConfig>,
    #[account(constraint = registry_config.authority == *authority.to_account_info().key @ ErrorCode::InsufficientAuthority)]
    pub authority: Signer<'info>,
    pub new_authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(ix: AddEntryIx)]
pub struct AddEntry<'info> {
    #[account(mut)]
    pub registry_config: Account<'info, RegistryConfig>,
    #[account(
        init,
        payer = creator,
        // extra space for future upgrades
        space = 1024,
        seeds = [registry_config.entry_seed.as_ref(), ix.primary_key.as_ref()],
        bump = ix.bump,
    )]
    pub entry: Account<'info, EntryData>,
    #[account(constraint = registry_config.permissionless_add || (registry_config.authority == *creator.to_account_info().key) @ ErrorCode::InsufficientAuthority)]
    pub creator: Signer<'info>,
    #[account(constraint = registry_config.authority == *authority.to_account_info().key @ ErrorCode::InsufficientAuthority)]
    pub authority: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VerifyEntry<'info> {
    #[account(mut)]
    pub registry_config: Account<'info, RegistryConfig>,
    #[account(mut)]
    pub entry: Account<'info, EntryData>,
    #[account(constraint = registry_config.authority == *authority.to_account_info().key @ ErrorCode::InsufficientAuthority)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UnverifyEntry<'info> {
    #[account(mut)]
    pub registry_config: Account<'info, RegistryConfig>,
    #[account(mut)]
    pub entry: Account<'info, EntryData>,
    #[account(constraint = registry_config.authority == *authority.to_account_info().key @ ErrorCode::InsufficientAuthority)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct RemoveEntry<'info> {
    #[account(mut)]
    pub registry_config: Account<'info, RegistryConfig>,
    #[account(mut)]
    pub entry: Account<'info, EntryData>,
    #[account(constraint = registry_config.authority == *authority.to_account_info().key || entry.creator == *authority.to_account_info().key @ ErrorCode::InsufficientAuthority)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(ix: AddSchemaIx)]
pub struct AddSchema<'info> {
    #[account(mut)]
    pub registry_config: Account<'info, RegistryConfig>,
    #[account(
        init,
        payer = creator,
        // extra space for future upgrades
        space = 1024,
        seeds = [b"schema".as_ref(), &[registry_config.schema_version + 1]],
        bump = ix.bump,
    )]
    pub schema: Account<'info, SchemaData>,
    #[account(constraint = registry_config.authority == *creator.to_account_info().key @ ErrorCode::InsufficientAuthority)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RemoveSchema<'info> {
    #[account(mut)]
    pub registry_config: Account<'info, RegistryConfig>,
    #[account(mut)]
    pub schema: Account<'info, SchemaData>,
    #[account(constraint = registry_config.authority == *authority.to_account_info().key @ ErrorCode::InsufficientAuthority)]
    pub authority: Signer<'info>,
}

///////////////// DATA /////////////////

#[account]
#[derive(Default)]
pub struct RegistryConfig{
    pub authority: Pubkey,
    pub entry_seed: String,
    pub permissionless_add: bool,
    pub schema_version: u8,
    pub add_fee: u64,
}

#[account]
pub struct EntryData {
    pub primary_key: Vec<u8>,
    pub creator: Pubkey,
    pub created_at: i64,
    pub is_verified: bool,
    pub verified_at: i64,
    pub schema_version: u8,
    pub data: Vec<u8>,
}

#[account]
pub struct SchemaData {
    pub created_at: i64,
    pub version: u8,
    pub data: Vec<u8>,
}

///////////////// ERRORS /////////////////

#[error]
pub enum ErrorCode {
    #[msg("User does not have authority to modify this registry entry")]
    InsufficientAuthority,
    #[msg("User does not have enough sol to add an entry to this registry")]
    InsufficientBalance,
}