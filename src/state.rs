use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

#[derive(Debug, Copy, Clone)]
pub struct SwapConfig {
    pub initialized: bool,
    pub swap_authority: Pubkey,
    pub index: u64,
    pub supply: u64,
    pub metadata_prefix: [u8; 32],
    pub symbol: [u8; 8],
    pub royalty_wallet: Pubkey,
    pub admin_account: Pubkey,
}

impl Sealed for SwapConfig {}

impl IsInitialized for SwapConfig {
    fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Pack for SwapConfig {
    const LEN: usize = 1 + 32 + 8 + 8 + 32 + 8 + 32 + 32;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, SwapConfig::LEN];

        let (
            initialized,
            swap_authority,
            index,
            supply,
            metadata_prefix,
            symbol,
            royalty_wallet,
            admin_account,
        ) = mut_array_refs![dst, 1, 32, 8, 8, 32, 8, 32, 32];

        initialized[0] = self.initialized as u8;
        swap_authority.copy_from_slice(&self.swap_authority.to_bytes());
        index.copy_from_slice(&self.index.to_le_bytes());
        supply.copy_from_slice(&self.supply.to_le_bytes());
        metadata_prefix.copy_from_slice(&self.metadata_prefix);
        symbol.copy_from_slice(&self.symbol);
        royalty_wallet.copy_from_slice(&self.royalty_wallet.to_bytes());
        admin_account.copy_from_slice(&self.admin_account.to_bytes());
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, solana_program::program_error::ProgramError> {
        let src = array_ref![src, 0, SwapConfig::LEN];

        let (
            initialized,
            swap_authority,
            index,
            supply,
            metadata_prefix,
            symbol,
            royalty_wallet,
            admin_account,
        ) = array_refs![src, 1, 32, 8, 8, 32, 8, 32, 32];

        let initialized = match initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        let swap_authority = Pubkey::new_from_array(*swap_authority);
        let index = u64::from_le_bytes(*index);
        let supply = u64::from_le_bytes(*supply);
        let metadata_prefix = metadata_prefix.clone();
        let symbol = symbol.clone();
        let royalty_wallet = Pubkey::new_from_array(*royalty_wallet);
        let admin_account = Pubkey::new_from_array(*admin_account);

        return Ok(Self {
            initialized,
            swap_authority,
            index,
            supply,
            metadata_prefix,
            symbol,
            royalty_wallet,
            admin_account,
        });
    }
}
