use anchor_lang::prelude::*;

#[account]
// #[derive(InitSpace)]
pub struct EscrowAccount {
    pub seller: Pubkey,  // 32
    pub buyer: Pubkey,  // 32
    pub amount: u64,    // 8
    pub status: TradeStatus,  // 1
    pub created_at: i64,  // 8
    pub expire_at: i64,  // 8
    pub bump: u8,  // 1
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum TradeStatus {
    Created,
    Funded,
    Completed,
    Cancelled,
}
