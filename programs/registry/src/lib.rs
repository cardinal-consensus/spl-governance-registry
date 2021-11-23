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
    
    pub fn register_program_instance(ctx: Context<RegisterInstance>, ix: RegisterInstanceIx) -> ProgramResult {
        let program_instance = &mut ctx.accounts.program_instance;
        program_instance.name = ix.name;
        program_instance.program_address = ix.program_address;
        program_instance.is_verified = false; // default to false
        Ok(())
    }

    pub fn verify_program_instance(ctx: Context<VerifyInstance>) -> ProgramResult {
        let program_instance = &mut ctx.accounts.program_instance;
        program_instance.is_verified = true;
        Ok(())
    }

    pub fn remove_program_instance(ctx: Context<RemoveInstance>) -> ProgramResult {
        ctx.accounts.program_instance.close(ctx.accounts.authority.to_account_info()).unwrap();
        Ok(())
    }
}

///////////////// Instructions /////////////////

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitIx {
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RegisterInstanceIx {
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
        space = 64,
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
#[instruction(ix: RegisterInstanceIx)]
pub struct RegisterInstance<'info> {
    #[account(
        init,
        payer = authority,
        space = 128,
        seeds = [b"governance-program".as_ref(), ix.seed.as_ref()],
        bump = ix.bump,
    )]
    pub program_instance: Account<'info, GovernanceProgramAccount>,
    // permissionless authority to add instances
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VerifyInstance<'info> {
    #[account(mut)]
    pub program_instance: Account<'info, GovernanceProgramAccount>,
    // TODO constraint this is the singleton registry context address? Maybe we can rely on account discriminator + owner check
    #[account(mut)]
    pub registry_context: Account<'info, RegistryContextAccount>,
    #[account(constraint = registry_context.authority == *authority.to_account_info().key @ ErrorCode::InsufficientAuthority)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct RemoveInstance<'info> {
    #[account(mut)]
    pub program_instance: Account<'info, GovernanceProgramAccount>,
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
    pub is_verified: bool,
}

#[error]
pub enum ErrorCode {
    #[msg("User does not have authority to modify this registry")]
    InsufficientAuthority,
}