use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

use crate::{
    assert::{assert_signer, assert_writeable},
    error::SwapError,
    instruction::{deserialize_instruction_data, InitializeArgs, SwapArgs, SwapInstruction},
    pda::find_mint_authority,
    state::SwapConfig,
    swap_pair::SwapPair,
};

mod initialize_swap;
mod swap_tokens;

pub fn process_instruction<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    match deserialize_instruction_data(instruction_data)? {
        SwapInstruction::Initialize(initialize_args) => {
            process_initialize_swap(initialize_args, program_id, accounts)
        }
        SwapInstruction::Swap(swap_args) => process_swap_tokens(swap_args, program_id, accounts),
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

fn process_swap_tokens<'a>(
    swap_args: SwapArgs,
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
) -> ProgramResult {
    let iter = &mut accounts.iter();
    let swap_source = next_account_info(iter)?;
    let swap_config_account = next_account_info(iter)?;
    let mint_account = next_account_info(iter)?;
    let swap_destination = next_account_info(iter)?;
    let destination_token_account = next_account_info(iter)?;
    let token_metadata_account = next_account_info(iter)?;
    let mint_authority = next_account_info(iter)?;
    let system_program = next_account_info(iter)?;
    let rent_sysvar = next_account_info(iter)?;
    let token_program = next_account_info(iter)?;
    let _associated_token_program = next_account_info(iter)?;
    let _metaplex_program = next_account_info(iter)?;
    let fee_payer = next_account_info(iter)?;
    let swap_authority = next_account_info(iter)?;
    let admin_account = next_account_info(iter)?;
    let royalty_wallet = next_account_info(iter)?;

    let mut source_tokens = Vec::<SwapPair>::new();

    msg!("Push source NFTs into one list");
    for _ in 0..swap_args.amount {
        let token_account = next_account_info(iter)?;
        msg!("Assert token account is writeable");
        assert_writeable(token_account)?;

        let mint = next_account_info(iter)?;
        msg!("Assert token mint is writeable");
        assert_writeable(mint)?;

        source_tokens.push(SwapPair {
            token_account,
            mint,
        });
    }

    msg!("End pushing accounts into list");

    msg!("Swap source checks");

    msg!("Assert swap source in signer");
    assert_signer(swap_source)?;

    msg!("Swap config checks");

    msg!("Assert swap config account is writeable");
    assert_writeable(swap_config_account)?;

    msg!("Mint account checks");

    msg!("Assert mint is signer");
    assert_signer(mint_account)?;
    msg!("Assert mint is writeable");
    assert_writeable(mint_account)?;

    msg!("Destination token account checks");

    msg!("Assert destination token account is writeable");
    assert_writeable(destination_token_account)?;

    msg!("Metadata account checks");

    msg!("Assert metadata writeable");
    assert_writeable(token_metadata_account)?;

    msg!("Mint authority checks");

    msg!("Assert mint authority is properly derived");
    let (mint_authority_pda, mint_authority_bump) =
        find_mint_authority(swap_config_account.key, program_id);

    if mint_authority_pda != *mint_authority.key {
        return Err(SwapError::PdaCheckFailed.into());
    }

    msg!("Unpack rent sysvar");
    let rent = Rent::from_account_info(rent_sysvar)?;

    msg!("Fee payer checks");

    msg!("Assert fee payer is signer");
    assert_signer(fee_payer)?;
    msg!("Assert fee payer is writeable");
    assert_writeable(fee_payer)?;

    msg!("Swap authority checks");

    msg!("Check swap authority signature");
    assert_signer(swap_authority)?;
    msg!("Check swap authority address");
    let swap_config = SwapConfig::unpack(*swap_config_account.data.borrow())?;

    if swap_config.swap_authority != *swap_authority.key {
        return Err(SwapError::AddressMismatch.into());
    }

    msg!("Admin account checks");

    msg!("Check admin account address");
    if swap_config.admin_account != *admin_account.key {
        return Err(SwapError::AddressMismatch.into());
    }

    msg!("Royalty wallet checks");
    if swap_config.royalty_wallet != *royalty_wallet.key {
        return Err(SwapError::AddressMismatch.into());
    }

    swap_tokens::logic_burn(swap_source, swap_destination, &source_tokens)?;
    swap_tokens::logic_mint(
        swap_config_account,
        mint_account,
        swap_destination,
        destination_token_account,
        token_metadata_account,
        mint_authority,
        rent_sysvar,
        rent,
        fee_payer,
        admin_account,
        royalty_wallet,
        mint_authority_bump,
        system_program,
        token_program,
    )?;

    Ok(())
}
