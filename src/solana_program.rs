use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Solana 智能合约示例 - 简单转账程序");

    let accounts_iter = &mut accounts.iter();
    let from_account = next_account_info(accounts_iter)?;
    let to_account = next_account_info(accounts_iter)?;

    // 验证账户所有者
    if from_account.owner != program_id {
        msg!("From account does not have the correct program id.");
        return Err(ProgramError::IncorrectProgramId);
    }
    if to_account.owner != program_id {
        msg!("To account does not have the correct program id.");
        return Err(ProgramError::IncorrectProgramId);
    }

    // 解析指令数据 (转账金额)
    let amount = instruction_data
        .get(..8)
        .and_then(|slice| slice.try_into().ok())
        .map(u64::from_le_bytes)
        .ok_or(ProgramError::InvalidInstructionData)?;

    msg!("转账金额: {}", amount);

    // 检查发送者余额
    let from_balance = from_account.lamports();
    if from_balance < amount {
        msg!("发送者余额不足");
        return Err(ProgramError::InsufficientFunds);
    }

    // 执行转账
    **from_account.lamports.borrow_mut() -= amount;
    **to_account.lamports.borrow_mut() += amount;

    msg!("转账成功! 从 {:?} 到 {:?}", from_account.key, to_account.key);

    Ok(())
}
