use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
};

use crate::{pda::MINT_AUTHORITY, state::SwapConfig};

pub fn logic<'a>(
    swap_config_account: &'a AccountInfo<'a>,
    mint_authority: &'a AccountInfo<'a>,
    fee_payer: &'a AccountInfo<'a>,
    program_id: &Pubkey,
    rent: Rent,
    swap_authority: Pubkey,
    supply: u64,
    metadata_prefix: [u8; 32],
    symbol: [u8; 8],
    royalty_wallet: Pubkey,
    admin_account: Pubkey,
    mint_authority_bump: u8,
) -> ProgramResult {
    let swap_config = SwapConfig::unpack_unchecked(*swap_config_account.data.borrow())?;

    msg!("Check if swap config already initialized");
    if swap_config.is_initialized() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    msg!("Fill config body");
    let swap_config = SwapConfig {
        initialized: true,
        admin_account,
        index: 0,
        supply,
        metadata_prefix,
        royalty_wallet,
        swap_authority,
        symbol,
    };

    msg!("Save config to chain");
    SwapConfig::pack(swap_config, *swap_config_account.data.borrow_mut())?;

    msg!("Initialize mint authority");
    let len = 0;
    let balance = rent.minimum_balance(len);
    let mint_authority_seeds = &[
        MINT_AUTHORITY.as_bytes(),
        swap_config_account.key.as_ref(),
        &[mint_authority_bump],
    ];

    msg!("Invoke system program to initialize");
    invoke_signed(
        &system_instruction::create_account(
            fee_payer.key,
            mint_authority.key,
            balance,
            len as u64,
            program_id,
        ),
        &[fee_payer.clone(), mint_authority.clone()],
        &[mint_authority_seeds],
    )?;

    Ok(())
}
