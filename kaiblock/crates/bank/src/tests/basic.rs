use bank::instruction::BankInstruction;
use bank::processor::process_instruction;
use bank::state::{Mint, Pubkey, TokenAccount};
use borsh::BorshSerialize;
use std::collections::HashMap;

fn rand_pubkey() -> Pubkey {
    let mut p = [0u8; 32];
    p[0] = rand::random::<u8>();
    p
}

#[test]
fn test_mint_and_transfer() {
    let program_id = [1u8; 32];
    let mint_key = vec![2u8; 32];
    let alice_key = vec![3u8; 32];
    let bob_key = vec![4u8; 32];

    let mut store: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();

    // create mint account and token accounts
    let mint = Mint::new(6, Some([9u8; 32]));
    store.insert(mint_key.clone(), mint.try_to_vec().unwrap());

    let alice_acct = TokenAccount::new([11u8;32], program_id);
    store.insert(alice_key.clone(), alice_acct.try_to_vec().unwrap());

    let bob_acct = TokenAccount::new([12u8;32], program_id);
    store.insert(bob_key.clone(), bob_acct.try_to_vec().unwrap());

    // mint 1000 to alice
    let instr = BankInstruction::MintTo { amount: 1000u128 }.try_to_vec().unwrap();
    // For this test we will call process_instruction with accounts map containing mint then alice
    let mut accounts_for_mint = HashMap::new();
    accounts_for_mint.insert(mint_key.clone(), store.get(&mint_key).unwrap().clone());
    accounts_for_mint.insert(alice_key.clone(), store.get(&alice_key).unwrap().clone());

    let signers = vec![[9u8;32]]; // mint authority
    process_instruction(&program_id, &mut accounts_for_mint, &instr, &signers).unwrap();

    // verify alice has 1000
    let alice_after = TokenAccount::try_from_slice(accounts_for_mint.get(&alice_key).unwrap()).unwrap();
    assert_eq!(alice_after.amount, 1000u128);

    // Now transfer 200 from alice to bob
    // put accounts in order source -> dest
    let mut accounts_for_transfer = HashMap::new();
    accounts_for_transfer.insert(alice_key.clone(), accounts_for_mint.get(&alice_key).unwrap().clone());
    accounts_for_transfer.insert(bob_key.clone(), store.get(&bob_key).unwrap().clone());

    let transfer_instr = BankInstruction::Transfer { amount: 200 }.try_to_vec().unwrap();
    // signer would be alice owner in a real runtime; omitted here
    process_instruction(&program_id, &mut accounts_for_transfer, &transfer_instr, &[]).unwrap();

    // check balances
    let alice_after2 = TokenAccount::try_from_slice(accounts_for_transfer.get(&alice_key).unwrap()).unwrap();
    let bob_after = TokenAccount::try_from_slice(accounts_for_transfer.get(&bob_key).unwrap()).unwrap();

    assert_eq!(alice_after2.amount, 800u128);
    assert_eq!(bob_after.amount, 200u128);
}
