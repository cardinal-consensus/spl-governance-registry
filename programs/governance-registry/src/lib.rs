use anchor_lang::prelude::*;
use anchor_lang::AccountsClose;

declare_id!("govHvVVCZsdJLynaFJdqEWBU9AbJ4aHYdZsWno114V9");
const CONFIG_PREFIX: &str = "registry-config";

#[program]
pub mod permissionless_verifiable_registry {
    use super::*;

    pub fn init(ctx: Context<Init>, ix: InitIx) -> ProgramResult {
        let registry_config = &mut ctx.accounts.registry_config;
        registry_config.authority = *ctx.accounts.authority.key;
        registry_config.realm_seed = ix.realm_seed;
        Ok(())
    }

    pub fn transfer_authority(ctx: Context<TransferAuthority>) -> ProgramResult {
        let registry_config = &mut ctx.accounts.registry_config;
        registry_config.authority = *ctx.accounts.new_authority.key;
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

    pub fn remove_entry(ctx: Context<RemoveEntry>) -> ProgramResult {
        ctx.accounts.entry.close(ctx.accounts.authority.to_account_info()).unwrap();
        Ok(())
    }
}

///////////////// Instructions /////////////////

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitIx {
    pub bump: u8,
    pub realm_seed: String,
    pub permissionless_add: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AddEntryIx {
    pub bump: u8,
    pub address: Pubkey,
    pub schema_version: u8,
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
        seeds = [CONFIG_PREFIX.as_ref()],
        bump = ix.bump,
    )]
    pub registry_config: Account<'info, RegistryConfig>,
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TransferAuthority<'info> {
    #[account(mut, seeds = [CONFIG_PREFIX.as_ref()], bump = registry_config.bump)]
    pub registry_config: ProgramAccount<'info, RegistryConfig>,
    #[account(constraint = registry_config.authority == *authority.to_account_info().key @ ErrorCode::InsufficientAuthority)]
    pub authority: Signer<'info>,
    pub new_authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(ix: AddEntryIx)]
pub struct AddEntry<'info> {
    #[account(mut, seeds = [CONFIG_PREFIX.as_ref()], bump = registry_config.bump)]
    pub registry_config: ProgramAccount<'info, RegistryConfig>,
    #[account(
        init,
        payer = creator,
        // extra space for future upgrades
        space = 1024,
        seeds = [registry_config.realm_seed.as_ref(), ix.address.as_ref()],
        bump = ix.bump,
    )]
    pub entry: ProgramAccount<'info, RealmData>,
    #[account(constraint = registry_config.permissionless_add || (registry_config.authority == *creator.to_account_info().key) @ ErrorCode::InsufficientAuthority)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VerifyEntry<'info> {
    #[account(mut, seeds = [CONFIG_PREFIX.as_ref()], bump = registry_config.bump)]
    pub registry_config: ProgramAccount<'info, RegistryConfig>,
    #[account(mut)]
    pub entry: ProgramAccount<'info, RealmData>,
    #[account(constraint = registry_config.authority == *authority.to_account_info().key @ ErrorCode::InsufficientAuthority)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UnverifyEntry<'info> {
    #[account(mut, seeds = [CONFIG_PREFIX.as_ref()], bump = registry_config.bump)]
    pub registry_config: ProgramAccount<'info, RegistryConfig>,
    #[account(mut)]
    pub entry: ProgramAccount<'info, RealmData>,
    #[account(constraint = registry_config.authority == *authority.to_account_info().key @ ErrorCode::InsufficientAuthority)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct RemoveEntry<'info> {
    #[account(mut, seeds = [CONFIG_PREFIX.as_ref()], bump = registry_config.bump)]
    pub registry_config: ProgramAccount<'info, RegistryConfig>,
    #[account(mut)]
    pub entry: ProgramAccount<'info, RealmData>,
    #[account(constraint = registry_config.authority == *authority.to_account_info().key || entry.creator == *authority.to_account_info().key @ ErrorCode::InsufficientAuthority)]
    pub authority: Signer<'info>,
}

///////////////// DATA /////////////////

#[account]
#[derive(Default)]
pub struct RegistryConfig{
    pub bump: u8,
    pub authority: Pubkey,
    pub realm_seed: String,
    pub permissionless_add: bool,
}

#[account]
pub struct RealmData {
    pub address: Pubkey,
    pub creator: Pubkey,
    pub created_at: i64,
    pub is_verified: bool,
    pub verified_at: i64,
    // schema below
    pub symbol: string,
    pub name: string,
    pub website: string
    pub program_id: Pubkey,
    pub program_version: u8,
    pub keywords: Vec<string>
    pub attributes: Vec<Vec<string>>,}

///////////////// ERRORS /////////////////

#[error]
pub enum ErrorCode {
    #[msg("User does not have authority to modify this registry entry")]
    InsufficientAuthority,
}
