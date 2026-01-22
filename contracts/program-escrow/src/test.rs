#![cfg(test)]

use super::*;
use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env, String};

#[test]
fn test_init_program() {
    let env = Env::default();
    let contract = ProgramEscrowContract;
    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let program_id = String::from_str(&env, "hackathon-2024-q1");

    let program_data = contract.init_program(&env, program_id.clone(), admin.clone(), token.clone());

    assert_eq!(program_data.program_id, program_id);
    assert_eq!(program_data.total_funds, 0);
    assert_eq!(program_data.remaining_balance, 0);
    assert_eq!(program_data.authorized_payout_key, admin);
    assert_eq!(program_data.token_address, token);
    assert_eq!(program_data.payout_history.len(), 0);
}

#[test]
#[should_panic(expected = "Program already initialized")]
fn test_init_program_twice() {
    let env = Env::default();
    let contract = ProgramEscrowContract;
    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let program_id = String::from_str(&env, "hackathon-2024-q1");

    contract.init_program(&env, program_id.clone(), admin.clone(), token.clone());
    contract.init_program(&env, program_id, admin, token); // Should panic
}

#[test]
fn test_lock_program_funds() {
    let env = Env::default();
    let contract = ProgramEscrowContract;
    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let program_id = String::from_str(&env, "hackathon-2024-q1");

    contract.init_program(&env, program_id, admin.clone(), token);
    let program_data = contract.lock_program_funds(&env, 50_000_000_000); // 50,000 XLM (in stroops)

    assert_eq!(program_data.total_funds, 50_000_000_000);
    assert_eq!(program_data.remaining_balance, 50_000_000_000);

    // Lock additional funds
    let program_data = contract.lock_program_funds(&env, 25_000_000_000);
    assert_eq!(program_data.total_funds, 75_000_000_000);
    assert_eq!(program_data.remaining_balance, 75_000_000_000);
}

#[test]
#[should_panic(expected = "Amount must be greater than zero")]
fn test_lock_program_funds_zero() {
    let env = Env::default();
    let contract = ProgramEscrowContract;
    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let program_id = String::from_str(&env, "hackathon-2024-q1");

    contract.init_program(&env, program_id, admin, token);
    contract.lock_program_funds(&env, 0);
}

#[test]
#[should_panic(expected = "Program not initialized")]
fn test_lock_program_funds_before_init() {
    let env = Env::default();
    let contract = ProgramEscrowContract;

    contract.lock_program_funds(&env, 10_000_000_000);
}

#[test]
fn test_single_payout() {
    let env = Env::default();
    let contract = ProgramEscrowContract;
    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let recipient = Address::generate(&env);
    let program_id = String::from_str(&env, "hackathon-2024-q1");

    contract.init_program(&env, program_id, admin.clone(), token);
    contract.lock_program_funds(&env, 50_000_000_000);

    // Set admin as invoker for authorization
    env.as_contract(&contract, || {
        env.set_invoker(&admin);
        let program_data = contract.single_payout(&env, recipient.clone(), 10_000_000_000);
        assert_eq!(program_data.remaining_balance, 40_000_000_000);
        assert_eq!(program_data.payout_history.len(), 1);
        
        let payout = program_data.payout_history.get(0).unwrap();
        assert_eq!(payout.recipient, recipient);
        assert_eq!(payout.amount, 10_000_000_000);
    });
}

#[test]
#[should_panic(expected = "Insufficient balance")]
fn test_single_payout_insufficient_balance() {
    let env = Env::default();
    let contract = ProgramEscrowContract;
    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let recipient = Address::generate(&env);
    let program_id = String::from_str(&env, "hackathon-2024-q1");

    contract.init_program(&env, program_id, admin.clone(), token);
    contract.lock_program_funds(&env, 10_000_000_000);

    env.as_contract(&contract, || {
        env.set_invoker(&admin);
        contract.single_payout(&env, recipient, 20_000_000_000); // Should panic
    });
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_single_payout_unauthorized() {
    let env = Env::default();
    let contract = ProgramEscrowContract;
    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let unauthorized = Address::generate(&env);
    let recipient = Address::generate(&env);
    let program_id = String::from_str(&env, "hackathon-2024-q1");

    contract.init_program(&env, program_id, admin, token);
    contract.lock_program_funds(&env, 10_000_000_000);

    // Try to payout as unauthorized user
    env.as_contract(&contract, || {
        env.set_invoker(&unauthorized);
        contract.single_payout(&env, recipient, 5_000_000_000); // Should panic
    });
}

#[test]
fn test_batch_payout() {
    let env = Env::default();
    let contract = ProgramEscrowContract;
    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let recipient1 = Address::generate(&env);
    let recipient2 = Address::generate(&env);
    let recipient3 = Address::generate(&env);
    let program_id = String::from_str(&env, "hackathon-2024-q1");

    contract.init_program(&env, program_id, admin.clone(), token);
    contract.lock_program_funds(&env, 100_000_000_000);

    let recipients = vec![
        &env,
        recipient1.clone(),
        recipient2.clone(),
        recipient3.clone(),
    ];
    let amounts = vec![&env, 10_000_000_000, 20_000_000_000, 15_000_000_000];

    env.as_contract(&contract, || {
        env.set_invoker(&admin);
        let program_data = contract.batch_payout(&env, recipients, amounts);
        assert_eq!(program_data.remaining_balance, 55_000_000_000);
        assert_eq!(program_data.payout_history.len(), 3);
    });
}

#[test]
#[should_panic(expected = "Recipients and amounts vectors must have the same length")]
fn test_batch_payout_mismatched_lengths() {
    let env = Env::default();
    let contract = ProgramEscrowContract;
    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let recipient1 = Address::generate(&env);
    let recipient2 = Address::generate(&env);
    let program_id = String::from_str(&env, "hackathon-2024-q1");

    contract.init_program(&env, program_id, admin.clone(), token);
    contract.lock_program_funds(&env, 100_000_000_000);

    let recipients = vec![&env, recipient1, recipient2];
    let amounts = vec![&env, 10_000_000_000]; // Mismatched length

    env.as_contract(&contract, || {
        env.set_invoker(&admin);
        contract.batch_payout(&env, recipients, amounts); // Should panic
    });
}

#[test]
#[should_panic(expected = "Cannot process empty batch")]
fn test_batch_payout_empty() {
    let env = Env::default();
    let contract = ProgramEscrowContract;
    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let program_id = String::from_str(&env, "hackathon-2024-q1");

    contract.init_program(&env, program_id, admin.clone(), token);
    contract.lock_program_funds(&env, 100_000_000_000);

    let recipients = vec![&env];
    let amounts = vec![&env];

    env.as_contract(&contract, || {
        env.set_invoker(&admin);
        contract.batch_payout(&env, recipients, amounts); // Should panic
    });
}

#[test]
fn test_get_program_info() {
    let env = Env::default();
    let contract = ProgramEscrowContract;
    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let program_id = String::from_str(&env, "hackathon-2024-q1");

    contract.init_program(&env, program_id.clone(), admin.clone(), token.clone());
    contract.lock_program_funds(&env, 50_000_000_000);

    let info = contract.get_program_info(&env);
    assert_eq!(info.program_id, program_id);
    assert_eq!(info.total_funds, 50_000_000_000);
    assert_eq!(info.remaining_balance, 50_000_000_000);
    assert_eq!(info.authorized_payout_key, admin);
    assert_eq!(info.token_address, token);
}

#[test]
fn test_get_remaining_balance() {
    let env = Env::default();
    let contract = ProgramEscrowContract;
    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let recipient = Address::generate(&env);
    let program_id = String::from_str(&env, "hackathon-2024-q1");

    contract.init_program(&env, program_id, admin.clone(), token);
    contract.lock_program_funds(&env, 50_000_000_000);
    
    assert_eq!(contract.get_remaining_balance(&env), 50_000_000_000);

    env.as_contract(&contract, || {
        env.set_invoker(&admin);
        contract.single_payout(&env, recipient, 10_000_000_000);
        assert_eq!(contract.get_remaining_balance(&env), 40_000_000_000);
    });
}

#[test]
#[should_panic(expected = "Program not initialized")]
fn test_get_info_before_init() {
    let env = Env::default();
    let contract = ProgramEscrowContract;

    contract.get_program_info(&env);
}
