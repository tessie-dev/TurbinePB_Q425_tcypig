use anchor_lang::prelude::*;

pub mod states;
pub mod instructions;
pub mod errors;

use instructions::*;

declare_id!("Aigh3VAp74HikYhD6ebdYuiq4GJ4ykpJtdvSG2sryYwn");

#[program]
pub mod capstone_safeswap {
    use super::*;

    pub fn create_escrow(ctx: Context<CreateEscrow>, amount: u64, expire_at: i64) -> Result<()> {
        let bump = ctx.bumps.escrow;
        ctx.accounts.init_escrow(amount, expire_at, bump)?;
        Ok(())
    }

    pub fn fund_escrow(ctx: Context<FundEscrow>) -> Result<()> {
        ctx.accounts.fund()?;
        Ok(())
    }

    pub fn complete_escrow(ctx: Context<CompleteEscrow>) -> Result<()> {
        ctx.accounts.complete()?;
        Ok(())
    }

    pub fn cancel_escrow(ctx: Context<CancelEscrow>) -> Result<()> {
        ctx.accounts.cancel()?;
        Ok(())
    }

    pub fn refund_escrow(ctx: Context<RefundEscrow>) -> Result<()> {
        ctx.accounts.refund()?;
        Ok(())
    }
}
