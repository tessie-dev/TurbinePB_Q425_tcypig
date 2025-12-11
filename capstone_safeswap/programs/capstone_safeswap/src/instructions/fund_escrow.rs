use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer};
use crate::states::{EscrowAccount, TradeStatus};
use crate::errors::EscrowError;

#[derive(Accounts)]
pub struct FundEscrow<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(mut)]
    pub escrow: Account<'info, EscrowAccount>,

    pub system_program: Program<'info, System>,
}


impl<'info> FundEscrow<'info> {
    pub fn fund(&mut self) -> Result<()> {
        let escrow = &mut self.escrow;

        require!(
            escrow.status == TradeStatus::Created,
            EscrowError::InvalidStatus
        );

        if escrow.buyer == Pubkey::default() {
            escrow.buyer = self.buyer.key();
        } else {
            require!(escrow.buyer == self.buyer.key(), EscrowError::WrongBuyer);
        }

        let buyer = &self.buyer;

        // buyer -> escrow
        let transfer_accounts = Transfer {
            from: buyer.to_account_info(),
            to: escrow.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(
            self.system_program.to_account_info(), 
            transfer_accounts
        );

        system_program::transfer(cpi_ctx, escrow.amount)?;

        // update escrow status
        escrow.status = crate::states::TradeStatus::Funded;

        Ok(())
    }
}