use arrayref::{array_ref, array_refs};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::error::SwapError;

const INITIALIZE_LEN: usize = 8 + 32 + 8 + 32 + 32 + 32;

pub struct InitializeArgs {
    pub supply: u64,
    pub metadata_prefix: [u8; 32],
    pub symbol: [u8; 8],
    pub swap_authority: Pubkey,
    pub royalty_wallet: Pubkey,
    pub admin_account: Pubkey,
}

const SWAP_LEN: usize = 1;

pub struct SwapArgs {
    pub amount: u8,
}

pub enum SwapInstruction {
    /// Required accounts:
    /// 0. `[signer, writeable]` - Fee payer
    /// 1. `[signer, writeable]` - Swap config account
    /// 2. `[writeable]` - Mint authority
    /// 3. `[]` - Rent sysvar
    /// 4. `[]` - System program
    Initialize(InitializeArgs),

    /// Required accounts:
    /// 0. `[signer]` - Swap source
    /// 1. `[writeable]` - Swap config account
    /// 2. `[signer, writeable]` - Mint account
    /// 3. `[writeable]` - Swap destination
    /// 4. `[writeable]` - Destination token account
    /// 5. `[writeable]` - Token metadata account
    /// 6. `[]` - Mint authority
    /// 7. `[]`. System program
    /// 8. `[]`. Rent sysvar
    /// 9. `[]`. Token program
    /// 10. `[]`. Associated token program
    /// 11. `[]`. Metaplex token metadata program
    /// 12. `[signer, writeable]` - Fee payer
    /// 13. `[signer]` - Swap authority
    /// 14. `[]` - Admin account
    /// 15. `[]` - Royalty wallet
    /// 16 + 2n. `[writeable]` - Source token accounts
    /// 17 + 2n. `[writeable]` - Source mint accounts
    Swap(SwapArgs),
}

fn parse_initialize_data(body: &[u8]) -> Result<InitializeArgs, ProgramError> {
    let body = array_ref![body, 0, INITIALIZE_LEN];

    let (supply, metadata_prefix, symbol, swap_authority, royalty_wallet, admin_account) =
        array_refs![body, 8, 32, 8, 32, 32, 32];

    let supply = u64::from_le_bytes(*supply);
    let metadata_prefix = *metadata_prefix;
    let symbol = *symbol;
    let swap_authority = Pubkey::new_from_array(*swap_authority);
    let royalty_wallet = Pubkey::new_from_array(*royalty_wallet);
    let admin_account = Pubkey::new_from_array(*admin_account);

    return Ok(InitializeArgs {
        supply,
        metadata_prefix,
        symbol,
        swap_authority,
        royalty_wallet,
        admin_account,
    });
}

fn parse_swap_data(body: &[u8]) -> Result<SwapArgs, ProgramError> {
    let body = array_ref![body, 0, SWAP_LEN];
    let amount = u8::from_le_bytes(*body);

    return Ok(SwapArgs { amount });
}

pub fn deserialize_instruction_data(
    instruction_data: &[u8],
) -> Result<SwapInstruction, ProgramError> {
    let (id, body) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidAccountData)?;

    match id {
        1 => Ok(SwapInstruction::Initialize(parse_initialize_data(body)?)),
        2 => Ok(SwapInstruction::Swap(parse_swap_data(body)?)),
        _ => Err(SwapError::BadInstructionId.into()),
    }
}
