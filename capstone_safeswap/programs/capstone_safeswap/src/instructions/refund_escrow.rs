use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer};
use crate::states::{EscrowAccount, TradeStatus};
use crate::errors::EscrowError;

#[derive(Accounts)]
pub struct RefundEscrow<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(mut)]
    pub escrow: Account<'info, EscrowAccount>,

    /// CHECK: vault PDA, system-owned
    #[account(
        mut,
        seeds = [b"vault", escrow.key().as_ref()],
        bump,
    )]
    pub vault: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}


impl<'info> RefundEscrow<'info> {
    pub fn refund(&mut self, vault_bump: u8) -> Result<()> {
        let escrow = &mut self.escrow;

        require!(
            escrow.buyer == self.buyer.key(),
            EscrowError::WrongBuyer
        );

        require!(
            escrow.status == TradeStatus::Funded,
            EscrowError::InvalidStatus
        );

        let buyer = &self.buyer;

        // escrow -> buyer
        let transfer_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: buyer.to_account_info(),
        };

        // PDA signer seeds
        let escrow_key = escrow.key();
        let seeds = &[
            b"vault".as_ref(),
            escrow_key.as_ref(),
            &[vault_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            self.system_program.to_account_info(), 
            transfer_accounts,
            signer_seeds,
        );

        system_program::transfer(cpi_ctx, escrow.amount)?;

        // update escrow status
        escrow.status = crate::states::TradeStatus::Cancelled;

        Ok(())
    }
}