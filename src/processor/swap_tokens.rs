use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, program::invoke};

use crate::swap_pair::SwapPair;

pub fn logic_burn<'a>(
    swap_source: &'a AccountInfo<'a>,
    source_tokens: &Vec<SwapPair<'a>>,
) -> ProgramResult {
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
                swap_source.key,
                swap_source.key,
                &[],
            )?,
            &[
                token.token_account.clone(),
                swap_source.clone(),
                swap_source.clone(),
            ],
        )?;
    }

    Ok(())
}

pub fn logic_mint<'a>() -> ProgramResult {
    todo!()
}
