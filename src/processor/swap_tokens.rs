use metaplex_token_metadata::state::Creator;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    rent::Rent,
    system_instruction,
};

use crate::{pda::MINT_AUTHORITY, state::SwapConfig, swap_pair::SwapPair};

pub fn logic_burn<'a>(
    swap_source: &'a AccountInfo<'a>,
    swap_destination: &'a AccountInfo<'a>,
    source_tokens: &Vec<SwapPair<'a>>,
) -> ProgramResult {
    msg!("Burn {} tokens", source_tokens.len());

    for token in source_tokens {
        invoke(
            &spl_token::instruction::burn(
                &spl_token::id(),
                token.token_account.key,
                token.mint.key,
                swap_source.key,
                &[],
                1,
            )?,
            &[
                token.token_account.clone(),
                token.mint.clone(),
                swap_source.clone(),
            ],
        )?;

        invoke(
            &spl_token::instruction::close_account(
                &spl_token::id(),
                &token.token_account.key,
                swap_destination.key,
                swap_source.key,
                &[],
            )?,
            &[
                token.token_account.clone(),
                swap_destination.clone(),
                swap_source.clone(),
            ],
        )?;
    }

    msg!("Burn successful");
    Ok(())
}

pub fn logic_mint<'a>(
    swap_config: &'a AccountInfo<'a>,
    mint_account: &'a AccountInfo<'a>,
    swap_destination: &'a AccountInfo<'a>,
    destination_token_account: &'a AccountInfo<'a>,
    metadata: &'a AccountInfo<'a>,
    mint_authority: &'a AccountInfo<'a>,
    rent_account: &'a AccountInfo<'a>,
    rent: Rent,
    fee_payer: &'a AccountInfo<'a>,
    admin_account: &'a AccountInfo<'a>,
    revenues: &'a AccountInfo<'a>,
    mint_authority_bump: u8,
    system_program: &'a AccountInfo<'a>,
    token_program: &'a AccountInfo<'a>,
) -> ProgramResult {
    let lamports = rent.minimum_balance(spl_token::state::Mint::LEN);

    msg!("Initialize account for mint");

    invoke(
        &system_instruction::create_account(
            fee_payer.key,
            mint_account.key,
            lamports,
            spl_token::state::Mint::LEN as u64,
            &spl_token::id(),
        ),
        &[fee_payer.clone(), mint_account.clone()],
    )?;

    msg!("Fill mint data");

    invoke(
        &spl_token::instruction::initialize_mint(
            &spl_token::id(),
            mint_account.key,
            mint_authority.key,
            None,
            0,
        )?,
        &[mint_account.clone(), rent_account.clone()],
    )?;

    msg!("Initialize user token account");

    invoke(
        &spl_associated_token_account::instruction::create_associated_token_account(
            fee_payer.key,
            swap_destination.key,
            mint_account.key,
        ),
        &[
            fee_payer.clone(),
            destination_token_account.clone(),
            swap_destination.clone(),
            mint_account.clone(),
            system_program.clone(),
            token_program.clone(),
        ],
    )?;

    let mut swap_data = SwapConfig::unpack(*swap_config.data.borrow())?;
    let symbol_str = str_from_u8_nul_utf8(&swap_data.symbol)
        .or(Err(ProgramError::InvalidAccountData))?
        .to_string();

    let uri_prefix = str_from_u8_nul_utf8(&swap_data.metadata_prefix)
        .or(Err(ProgramError::InvalidAccountData))?
        .to_string();

    let creators = vec![
        Creator {
            address: *mint_authority.key,
            verified: true,
            share: 0,
        },
        Creator {
            address: *revenues.key,
            verified: false,
            share: 100,
        },
    ];

    let mint_authority_seed = &[
        MINT_AUTHORITY.as_bytes(),
        swap_config.key.as_ref(),
        &[mint_authority_bump],
    ];

    msg!("Initialize metadata");

    invoke_signed(
        &metaplex_token_metadata::instruction::create_metadata_accounts(
            metaplex_token_metadata::id(),
            *metadata.key,
            *mint_account.key,
            *mint_authority.key,
            *fee_payer.key,
            *mint_authority.key,
            symbol_str.clone() + &format!(" #{}", swap_data.index),
            symbol_str,
            uri_prefix + &format!("{}.json", swap_data.index),
            Some(creators),
            1000,
            false,
            true,
        ),
        &[
            metadata.clone(),
            mint_account.clone(),
            mint_authority.clone(),
            fee_payer.clone(),
            mint_authority.clone(),
            system_program.clone(),
            rent_account.clone(),
        ],
        &[mint_authority_seed],
    )?;

    msg!("Mint to user");

    invoke_signed(
        &spl_token::instruction::mint_to(
            &spl_token::id(),
            mint_account.key,
            destination_token_account.key,
            mint_authority.key,
            &[],
            1,
        )?,
        &[
            mint_account.clone(),
            destination_token_account.clone(),
            mint_authority.clone(),
        ],
        &[mint_authority_seed],
    )?;

    msg!("Update metadata");

    invoke_signed(
        &metaplex_token_metadata::instruction::update_metadata_accounts(
            metaplex_token_metadata::id(),
            *metadata.key,
            *mint_authority.key,
            Some(*admin_account.key),
            None,
            Some(true),
        ),
        &[metadata.clone(), mint_authority.clone()],
        &[mint_authority_seed],
    )?;

    msg!("Revoke mint authority");

    invoke_signed(
        &spl_token::instruction::set_authority(
            &spl_token::id(),
            mint_account.key,
            None,
            spl_token::instruction::AuthorityType::MintTokens,
            mint_authority.key,
            &[],
        )?,
        &[mint_account.clone(), mint_authority.clone()],
        &[mint_authority_seed],
    )?;

    msg!("Write changes to program accounts");
    swap_data.index += 1;
    SwapConfig::pack(swap_data, *swap_config.data.borrow_mut())?;

    Ok(())
}

fn str_from_u8_nul_utf8(utf8_src: &[u8]) -> Result<&str, std::str::Utf8Error> {
    let nul_range_end = utf8_src
        .iter()
        .position(|&c| c == b'\0')
        .unwrap_or(utf8_src.len()); // default to length if no `\0` present
    ::std::str::from_utf8(&utf8_src[0..nul_range_end])
}
