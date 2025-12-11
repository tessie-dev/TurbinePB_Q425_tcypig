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

    pub system_program: Program<'info, System>,
}


impl<'info> RefundEscrow<'info> {
    pub fn refund(&mut self) -> Result<()> {
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
            from: escrow.to_account_info(),
            to: buyer.to_account_info(),
        };

        // PDA signer seeds
        let seeds = &[
            b"escrow".as_ref(),
            escrow.seller.as_ref(),
            &[escrow.bump],
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