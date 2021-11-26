use anchor_lang::prelude::*;
use anchor_lang::AccountsClose;

declare_id!("govHvVVCZsdJLynaFJdqEWBU9AbJ4aHYdZsWno114V9");

#[program]
pub mod governanceregistry {
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
        entry.name = ix.name;
        entry.program_address = ix.program_address;
        entry.created_at = timestamp;
        entry.creator = *ctx.accounts.authority.key;
        entry.is_verified = false; // default to false
        Ok(())
    }

    pub fn verify_entry(ctx: Context<VerifyEntry>) -> ProgramResult {
        let clock = Clock::get().unwrap();
        let timestamp = clock.unix_timestamp;
        let entry = &mut ctx.accounts.entry;
        entry.is_verified = true;
        entry.verified_at = timestamp;
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
    pub name: String,
    pub program_address: Pubkey,
    pub bump: u8,
    pub seed: [u8; 32],
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
    #[account(
        init,
        payer = authority,
        // extra space for future upgrades
        space = 256,
        seeds = [b"governance-program".as_ref(), ix.seed.as_ref()],
        bump = ix.bump,
    )]
    pub entry: Account<'info, GovernanceProgramAccount>,
    // permissionless authority to add entries
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VerifyEntry<'info> {
    #[account(mut)]
    pub entry: Account<'info, GovernanceProgramAccount>,
    // TODO constraint this is the singleton registry context address? Maybe we can rely on account discriminator + owner check
    #[account(mut)]
    pub registry_context: Account<'info, RegistryContextAccount>,
    #[account(constraint = registry_context.authority == *authority.to_account_info().key @ ErrorCode::InsufficientAuthority)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct RemoveEntry<'info> {
    #[account(mut)]
    pub entry: Account<'info, GovernanceProgramAccount>,
    // TODO constraint this is the singleton registry context address? Maybe we can rely on account discriminator + owner check
    #[account(mut)]
    pub registry_context: Account<'info, RegistryContextAccount>,
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
pub struct GovernanceProgramAccount {
    pub name: String,
    pub program_address: Pubkey,
    // pub additional_data_url: String,
    pub is_verified: bool,
    pub verified_at: i64,
    pub created_at: i64,
    pub creator: Pubkey,
}

#[error]
pub enum ErrorCode {
    #[msg("User does not have authority to modify this registry")]
    InsufficientAuthority,
}