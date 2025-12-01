use anchor_lang::prelude::*;

declare_id!("Aigh3VAp74HikYhD6ebdYuiq4GJ4ykpJtdvSG2sryYwn");

#[program]
pub mod capstone_safeswap {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
