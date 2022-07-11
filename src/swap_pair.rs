use solana_program::account_info::AccountInfo;

pub struct SwapPair<'a> {
    pub token_account: &'a AccountInfo<'a>,
    pub mint: &'a AccountInfo<'a>
}