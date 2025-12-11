use anchor_lang::prelude::*;
use crate::states::EscrowAccount;

#[derive(Accounts)]
pub struct CreateEscrow<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(
        init,
        payer = seller,
        space = 8 + 32 + 32 + 8 + 1 + 8 + 8 + 1,
        seeds = [b"escrow", seller.key().as_ref()],
        bump,
    )]
    pub escrow: Account<'info, EscrowAccount>,

    pub system_program: Program<'info, System>,
}


// create the trade but with the empty buyer field
impl<'info> CreateEscrow<'info> {
    pub fn init_escrow(&mut self, amount: u64, expire_at: i64, bump: u8) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;

        self.escrow.set_inner(EscrowAccount {
            seller: self.seller.key(),
            buyer: Pubkey::default(),
            amount,
            status: crate::states::TradeStatus::Created,
            created_at: now,
            expire_at,
            bump,
        });

        Ok(())
    }
}