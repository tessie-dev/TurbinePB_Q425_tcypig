use anchor_lang::prelude::*;

pub mod states;
pub mod instructions;
pub mod errors;

use instructions::*;

declare_id!("HNKvPBE1hit4vzsrmpLVfAggE8Hg3RXhvAAy2DVEm1Ue");

#[program]
pub mod capstone_safeswap {
    use super::*;

    pub fn create_escrow(ctx: Context<CreateEscrow>, listing_id: u64, amount: u64, expire_at: i64) -> Result<()> {
        let bump = ctx.bumps.escrow;
        ctx.accounts.init_escrow(amount, expire_at, bump, listing_id)?;
        Ok(())
    }

    pub fn fund_escrow(ctx: Context<FundEscrow>, listing_id: u64) -> Result<()> {
        ctx.accounts.fund()?;
        Ok(())
    }

    pub fn complete_escrow(ctx: Context<CompleteEscrow>, listing_id: u64) -> Result<()> {
        let vault_bump = ctx.bumps.vault;
        ctx.accounts.complete(vault_bump)?;
        Ok(())
    }

    pub fn cancel_escrow(ctx: Context<CancelEscrow>, listing_id: u64) -> Result<()> {
        ctx.accounts.cancel()?;
        Ok(())
    }

    pub fn refund_escrow(ctx: Context<RefundEscrow>, listing_id: u64) -> Result<()> {
        let vault_bump = ctx.bumps.vault;
        ctx.accounts.refund(vault_bump)?;
        Ok(())
    }
}
