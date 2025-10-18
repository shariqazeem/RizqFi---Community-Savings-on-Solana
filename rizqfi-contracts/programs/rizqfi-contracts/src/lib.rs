use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};

declare_id!("sv1jTY2Rs2H7fXVVj3a6WoP6GD3X2j5tHwYkdfZQ9oP");

#[program]
pub mod rizqfi_contracts {
    use super::*;

    // Create a new savings committee
    pub fn create_committee(
        ctx: Context<CreateCommittee>,
        name: String,
        monthly_contribution: u64,
        max_members: u8,
        payout_mode: PayoutMode,
        rotation_type: RotationType,
        frequency: Frequency,
        start_date: i64,
    ) -> Result<()> {
        let committee = &mut ctx.accounts.committee;
        
        require!(max_members >= 2 && max_members <= 20, ErrorCode::InvalidMemberCount);
        require!(monthly_contribution > 0, ErrorCode::InvalidContribution);
        
        committee.authority = ctx.accounts.authority.key();
        committee.name = name;
        committee.monthly_contribution = monthly_contribution;
        committee.max_members = max_members;
        committee.current_members = 0;
        committee.current_round = 0;
        committee.deposits_this_round = 0;
        committee.is_active = false;
        committee.payout_mode = payout_mode;
        committee.rotation_type = rotation_type;
        committee.frequency = frequency;
        committee.vault = ctx.accounts.vault.key();
        committee.created_at = Clock::get()?.unix_timestamp;
        committee.start_date = start_date;
        committee.next_round_date = start_date;
        committee.next_payout_recipient = 0;
        committee.members_who_received_payout = vec![];
        committee.phase = CommitteePhase::Joining;
        
        msg!("Committee created: {}", committee.name);
        Ok(())
    }

    // Join an existing committee
    pub fn join_committee(ctx: Context<JoinCommittee>) -> Result<()> {
        let committee = &mut ctx.accounts.committee;
        let member = &mut ctx.accounts.member;
        
        require!(committee.phase == CommitteePhase::Joining, ErrorCode::NotInJoiningPhase);
        require!(committee.current_members < committee.max_members, ErrorCode::CommitteeFull);
        
        member.committee = committee.key();
        member.authority = ctx.accounts.authority.key();
        member.token_account = ctx.accounts.member_token_account.key();
        member.total_contributed = 0;
        member.rounds_participated = 0;
        member.has_received_payout = false;
        member.missed_payments = 0;
        member.joined_at = Clock::get()?.unix_timestamp;
        member.is_active = true;
        member.has_deposited_current_round = false;
        
        committee.current_members += 1;
        
        if committee.current_members == committee.max_members {
            committee.phase = CommitteePhase::Deposit;
            committee.is_active = true;
            msg!("Committee full! Moving to Deposit phase.");
        }
        
        msg!("Member joined committee: {} ({}/{})", committee.name, committee.current_members, committee.max_members);
        Ok(())
    }

    // Make monthly contribution (deposit USDC)
    pub fn contribute(ctx: Context<Contribute>) -> Result<()> {
        let committee = &mut ctx.accounts.committee;
        let member = &mut ctx.accounts.member;
        
        require!(committee.is_active, ErrorCode::CommitteeInactive);
        require!(committee.phase == CommitteePhase::Deposit, ErrorCode::NotInDepositPhase);
        require!(member.is_active, ErrorCode::MemberInactive);
        require!(!member.has_deposited_current_round, ErrorCode::AlreadyDepositedThisRound);
        
        let cpi_accounts = Transfer {
            from: ctx.accounts.member_token_account.to_account_info(),
            to: ctx.accounts.committee_vault.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        token::transfer(cpi_ctx, committee.monthly_contribution)?;
        
        member.total_contributed += committee.monthly_contribution;
        member.rounds_participated += 1;
        member.has_deposited_current_round = true;
        
        committee.deposits_this_round += 1;
        
        msg!("Contribution received: {} USDC from {} ({}/{})", 
            committee.monthly_contribution, 
            member.authority,
            committee.deposits_this_round,
            committee.current_members
        );
        
        if committee.deposits_this_round == committee.current_members {
            committee.phase = CommitteePhase::Payout;
            msg!("All members deposited! Moving to Payout phase.");
        }
        
        Ok(())
    }

    // Distribute payout to designated member
    pub fn distribute_payout(ctx: Context<DistributePayout>) -> Result<()> {
        let committee = &mut ctx.accounts.committee;
        let recipient_member = &mut ctx.accounts.recipient_member;
        
        require!(committee.is_active, ErrorCode::CommitteeInactive);
        require!(committee.phase == CommitteePhase::Payout, ErrorCode::NotInPayoutPhase);
        require!(!recipient_member.has_received_payout, ErrorCode::AlreadyReceivedPayout);
        require!(committee.deposits_this_round == committee.current_members, ErrorCode::NotAllDepositsReceived);
        
        let total_payout = committee.monthly_contribution * committee.current_members as u64;
        let authority_key = committee.authority;
        let bump = ctx.bumps.committee;
        
        let seeds = &[
            b"committee",
            authority_key.as_ref(),
            &[bump],
        ];
        let signer = &[&seeds[..]];
        
        let cpi_accounts = Transfer {
            from: ctx.accounts.committee_vault.to_account_info(),
            to: ctx.accounts.recipient_token_account.to_account_info(),
            authority: committee.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        
        token::transfer(cpi_ctx, total_payout)?;
        
        recipient_member.has_received_payout = true;
        recipient_member.has_deposited_current_round = false;
        committee.members_who_received_payout.push(recipient_member.authority);
        committee.current_round += 1;
        
        let frequency_seconds = match committee.frequency {
            Frequency::Weekly => 7 * 24 * 60 * 60,
            Frequency::Biweekly => 14 * 24 * 60 * 60,
            Frequency::Monthly => 30 * 24 * 60 * 60,
        };
        
        committee.next_round_date = Clock::get()?.unix_timestamp + frequency_seconds;
        committee.deposits_this_round = 0;
        
        if committee.members_who_received_payout.len() as u8 == committee.current_members {
            committee.is_active = false;
            committee.phase = CommitteePhase::Completed;
            msg!("Committee completed! All members received payouts.");
        } else {
            committee.phase = CommitteePhase::Deposit;
        }
        
        msg!("Payout distributed: {} USDC to {}", total_payout, recipient_member.authority);
        msg!("Next round starts at: {}", committee.next_round_date);
        Ok(())
    }

    // Reset member deposit status for new round
    pub fn reset_member_deposit_status(ctx: Context<ResetMemberDepositStatus>) -> Result<()> {
        let committee = &ctx.accounts.committee;
        let member = &mut ctx.accounts.member;
        
        require!(committee.is_active, ErrorCode::CommitteeInactive);
        require!(committee.phase == CommitteePhase::Deposit, ErrorCode::NotInDepositPhase);
        
        member.has_deposited_current_round = false;
        
        msg!("Member {} deposit status reset for round {}", member.authority, committee.current_round);
        Ok(())
    }

    // Handle missed payment
    pub fn record_missed_payment(ctx: Context<RecordMissedPayment>) -> Result<()> {
        let member = &mut ctx.accounts.member;
        
        member.missed_payments += 1;
        
        if member.missed_payments >= 2 {
            member.is_active = false;
            msg!("Member {} marked inactive due to missed payments", member.authority);
        }
        
        Ok(())
    }

    // Close committee and return remaining funds
    pub fn close_committee(ctx: Context<CloseCommittee>) -> Result<()> {
        let committee = &ctx.accounts.committee;
        
        require!(committee.phase == CommitteePhase::Completed, ErrorCode::CommitteeNotCompleted);
        
        let vault_balance = ctx.accounts.committee_vault.amount;
        
        if vault_balance > 0 {
            let authority_key = committee.authority;
            let bump = ctx.bumps.committee;
            
            let seeds = &[
                b"committee",
                authority_key.as_ref(),
                &[bump],
            ];
            let signer = &[&seeds[..]];
            
            let cpi_accounts = Transfer {
                from: ctx.accounts.committee_vault.to_account_info(),
                to: ctx.accounts.authority_token_account.to_account_info(),
                authority: committee.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            
            token::transfer(cpi_ctx, vault_balance)?;
        }
        
        msg!("Committee closed, remaining funds returned");
        Ok(())
    }
}

// Account structures
#[derive(Accounts)]
#[instruction(name: String)]
pub struct CreateCommittee<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Committee::INIT_SPACE,
        seeds = [b"committee", authority.key().as_ref()],
        bump
    )]
    pub committee: Account<'info, Committee>,
    
    #[account(
        init,
        payer = authority,
        token::mint = usdc_mint,
        token::authority = committee,
        seeds = [b"vault", committee.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, TokenAccount>,
    
    pub usdc_mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct JoinCommittee<'info> {
    #[account(mut)]
    pub committee: Account<'info, Committee>,
    
    #[account(
        init,
        payer = authority,
        space = 8 + Member::INIT_SPACE,
        seeds = [b"member", committee.key().as_ref(), authority.key().as_ref()],
        bump
    )]
    pub member: Account<'info, Member>,
    
    pub member_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Contribute<'info> {
    #[account(mut)]
    pub committee: Account<'info, Committee>,
    
    #[account(
        mut,
        seeds = [b"member", committee.key().as_ref(), authority.key().as_ref()],
        bump
    )]
    pub member: Account<'info, Member>,
    
    #[account(mut)]
    pub member_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        address = committee.vault
    )]
    pub committee_vault: Account<'info, TokenAccount>,
    
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct DistributePayout<'info> {
    #[account(
        mut,
        seeds = [b"committee", committee.authority.as_ref()],
        bump
    )]
    pub committee: Account<'info, Committee>,
    
    #[account(mut)]
    pub recipient_member: Account<'info, Member>,
    
    #[account(
        mut,
        address = committee.vault
    )]
    pub committee_vault: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub recipient_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ResetMemberDepositStatus<'info> {
    pub committee: Account<'info, Committee>,
    
    #[account(
        mut,
        seeds = [b"member", committee.key().as_ref(), member.authority.as_ref()],
        bump
    )]
    pub member: Account<'info, Member>,
}

#[derive(Accounts)]
pub struct RecordMissedPayment<'info> {
    #[account(mut)]
    pub member: Account<'info, Member>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct CloseCommittee<'info> {
    #[account(
        mut,
        seeds = [b"committee", authority.key().as_ref()],
        bump,
        close = authority
    )]
    pub committee: Account<'info, Committee>,
    
    #[account(
        mut,
        address = committee.vault
    )]
    pub committee_vault: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub authority_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}

// Rest of your code (Committee, Member, enums, error codes) stays the same...

// Data structures
#[account]
#[derive(InitSpace)]
pub struct Committee {
    pub authority: Pubkey,
    #[max_len(50)]
    pub name: String,
    pub monthly_contribution: u64,
    pub max_members: u8,
    pub current_members: u8,
    pub current_round: u32,
    pub deposits_this_round: u8,
    pub next_payout_recipient: u8,
    pub is_active: bool,
    pub payout_mode: PayoutMode,
    pub rotation_type: RotationType,
    pub frequency: Frequency,
    pub phase: CommitteePhase,
    pub vault: Pubkey,
    pub created_at: i64,
    pub start_date: i64,
    pub next_round_date: i64,
    #[max_len(20)]
    pub members_who_received_payout: Vec<Pubkey>,
}

#[account]
#[derive(InitSpace)]
pub struct Member {
    pub committee: Pubkey,
    pub authority: Pubkey,
    pub token_account: Pubkey,
    pub total_contributed: u64,
    pub rounds_participated: u32,
    pub has_received_payout: bool,
    pub has_deposited_current_round: bool,
    pub missed_payments: u8,
    pub joined_at: i64,
    pub is_active: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum PayoutMode {
    Auto,
    Manual,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum RotationType {
    Fixed,
    Random,
    Bidding,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum Frequency {
    Weekly,
    Biweekly,
    Monthly,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum CommitteePhase {
    Joining,    // Members are still joining
    Deposit,    // Collecting deposits for current round
    Payout,     // Ready to distribute payout
    Completed,  // All rounds finished
}

// Error codes
#[error_code]
pub enum ErrorCode {
    #[msg("Committee must have 2-20 members")]
    InvalidMemberCount,
    #[msg("Monthly contribution must be greater than 0")]
    InvalidContribution,
    #[msg("Committee is not active")]
    CommitteeInactive,
    #[msg("Committee is full")]
    CommitteeFull,
    #[msg("Member is not active")]
    MemberInactive,
    #[msg("Member has already received payout")]
    AlreadyReceivedPayout,
    #[msg("Invalid payout mode for this operation")]
    InvalidPayoutMode,
    #[msg("Member has not contributed this round")]
    NoContribution,
    #[msg("Committee is not in joining phase")]
    NotInJoiningPhase,
    #[msg("Committee is not in deposit phase")]
    NotInDepositPhase,
    #[msg("Committee is not in payout phase")]
    NotInPayoutPhase,
    #[msg("Member already deposited this round")]
    AlreadyDepositedThisRound,
    #[msg("Not all deposits received")]
    NotAllDepositsReceived,
    #[msg("Committee not completed yet")]
    CommitteeNotCompleted,
}