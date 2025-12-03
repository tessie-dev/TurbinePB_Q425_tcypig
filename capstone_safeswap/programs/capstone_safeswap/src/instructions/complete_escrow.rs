use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer};
use crate::states::{EscrowAccount, TradeStatus};
use crate::errors::EscrowError;

// Buyer confirms receipt: escrow (PDA) releases funds to the seller.
// escrow -> seller

#[derive(Accounts)]
pub struct CompleteEscrow<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(mut)]
    pub seller: SystemAccount<'info>,

    #[account(mut)]
    pub escrow: Account<'info, EscrowAccount>,

    pub system_program: Program<'info, System>,
}


impl<'info> CompleteEscrow<'info> {
    pub fn complete(&mut self) -> Result<()> {
        require!(
            self.escrow.buyer == self.buyer.key(),
            EscrowError::WrongBuyer
        );

        require!(
            self.escrow.seller == self.seller.key(),
            EscrowError::WrongSeller
        );

        require!(
            self.escrow.status == TradeStatus::Funded,
            EscrowError::InvalidStatus
        );

        let seller = &self.seller;
        let escrow = &mut self.escrow;

        // escrow -> seller
        let transfer_accounts = Transfer {
            from: escrow.to_account_info(),
            to: seller.to_account_info(),
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
        escrow.status = crate::states::TradeStatus::Completed;

        Ok(())
    }
}
