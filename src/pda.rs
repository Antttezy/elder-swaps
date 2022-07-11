use solana_program::pubkey::Pubkey;

pub const MINT_AUTHORITY: &str = "mint_authority";

pub fn find_mint_authority(swap_config_address: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[MINT_AUTHORITY.as_bytes(), swap_config_address.as_ref()],
        program_id,
    )
}
