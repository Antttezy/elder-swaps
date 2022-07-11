use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

use crate::{
    assert::{assert_signer, assert_writeable},
    error::SwapError,
    instruction::{deserialize_instruction_data, InitializeArgs, SwapInstruction},
    pda::find_mint_authority,
};

mod initialize_swap;

pub fn process_instruction<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    match deserialize_instruction_data(instruction_data)? {
        SwapInstruction::Initialize(initialize_args) => {
            process_initialize_swap(initialize_args, program_id, accounts)
        }
        SwapInstruction::Swap(_) => todo!(),
    }
}

fn process_initialize_swap<'a>(
    initialize_args: InitializeArgs,
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
) -> ProgramResult {
    let iter = &mut accounts.iter();
    let fee_payer = next_account_info(iter)?;
    let swap_config_account = next_account_info(iter)?;
    let mint_authority = next_account_info(iter)?;
    let rent_sysvar = next_account_info(iter)?;
    let _system_program = next_account_info(iter)?;

    msg!("Fee payer checks");

    msg!("Assert signer");
    assert_signer(fee_payer)?;
    msg!("Assert writeable");
    assert_writeable(fee_payer)?;

    msg!("Swap config check");

    msg!("Assert signer");
    assert_signer(swap_config_account)?;
    msg!("Assert writeable");
    assert_writeable(swap_config_account)?;

    msg!("Mint authority check");

    msg!("Assert writeable");
    assert_writeable(mint_authority)?;
    msg!("Assert derived");

    let (mint_authority_pda, mint_authority_bump) =
        find_mint_authority(swap_config_account.key, program_id);

    if mint_authority_pda != *mint_authority.key {
        return Err(SwapError::PdaCheckFailed.into());
    }

    msg!("Unpack rent sysvar");
    let rent = Rent::from_account_info(rent_sysvar)?;

    initialize_swap::logic(
        swap_config_account,
        mint_authority,
        fee_payer,
        program_id,
        rent,
        initialize_args.swap_authority,
        initialize_args.supply,
        initialize_args.metadata_prefix,
        initialize_args.symbol,
        initialize_args.royalty_wallet,
        initialize_args.admin_account,
        mint_authority_bump,
    )?;

    Ok(())
}
