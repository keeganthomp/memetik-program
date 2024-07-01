import * as anchor from '@coral-xyz/anchor';
import { Memetik } from '../target/types/memetik';
import { getLogs } from '@solana-developers/helpers';
import { LAMPORTS_PER_SOL } from '@solana/web3.js';

// Configure the client to use the local cluster.
anchor.setProvider(anchor.AnchorProvider.env());
const program = anchor.workspace.Memetik as anchor.Program<Memetik>;
const provider = anchor.getProvider();

export const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey(
  'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s'
);
export const SOL_MINT = new anchor.web3.PublicKey(
  'So11111111111111111111111111111111111111112'
);

export const getLamports = (amount: number) => {
  return amount * LAMPORTS_PER_SOL;
};

export const getSol = (lamports: number) => {
  return lamports / LAMPORTS_PER_SOL;
};

export const getSPLBalance = async (tokenAccount) => {
  try {
    const info = await provider.connection.getTokenAccountBalance(
      tokenAccount
    );
    if (info.value.uiAmount == null) return 0;
    return info.value.uiAmount * LAMPORTS_PER_SOL;
  } catch (err) {
    return 0;
  }
};

const sleep = (ms: number) => {
  console.log(`Waiting for ${ms / 1000} seconds...`);
  return new Promise((resolve) => setTimeout(resolve, ms));
};

export const getSOLBalance = async (account: anchor.web3.PublicKey) => {
  const balance = await provider.connection.getBalance(account);
  return balance;
};

export const logTxnInfo = async (
  txn: anchor.web3.TransactionSignature
) => {
  await waitForTxnConfrimation(txn);
  const logs = await getLogs(provider.connection, txn);
  console.log('Transaction logs:', logs);
};

export const waitUntilTime = async (targetTimestamp) => {
  const BUFFER = 11000;
  const currentTime = Date.now();
  const delay = targetTimestamp + BUFFER - currentTime;
  console.log('Waiting....');
  console.log('current time:', currentTime);
  console.log('target time:', targetTimestamp);
  if (delay <= 0) {
    // If the target time is in the past or immediate future, resolve immediately
    return Promise.resolve();
  }
  return new Promise((resolve) => {
    setTimeout(resolve, delay);
  });
};

const waitForTxnConfrimation = async (
  tx: anchor.web3.TransactionSignature
) => {
  console.log('Waiting for transaction to be confirmed...');
  const confirmedTxn = await provider.connection.getTransaction(tx, {
    commitment: 'confirmed',
    maxSupportedTransactionVersion: 0,
  });
  console.log('Transaction confirmed!');
  return confirmedTxn;
};

export const fundSol = async (
  receiver: anchor.web3.PublicKey,
  solAmt = 80000
) => {
  const amtInLamports = solAmt * anchor.web3.LAMPORTS_PER_SOL;
  const sig = await provider.connection.requestAirdrop(
    receiver,
    amtInLamports
  );
  await provider.connection.confirmTransaction(sig, 'confirmed');
};

export const getMintPDA = (ticker: string) => {
  const MINT_SEED_CONSTANT = 'pool_mint';
  const seeds = [Buffer.from(MINT_SEED_CONSTANT), Buffer.from(ticker)];
  const [mintPDA] = anchor.web3.PublicKey.findProgramAddressSync(
    seeds,
    program.programId
  );
  return mintPDA;
};

export const getMetadataPDA = (mint: anchor.web3.PublicKey) => {
  const METADATA_SEED_CONSTANT = 'metadata';
  const [metadataAddress] = anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from(METADATA_SEED_CONSTANT),
      TOKEN_METADATA_PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
    ],
    TOKEN_METADATA_PROGRAM_ID
  );
  return metadataAddress;
};

export const getPoolPDA = (ticker: string) => {
  const POOL_BONDING_SEED_CONSTANT = 'pool';
  const seeds = [
    Buffer.from(POOL_BONDING_SEED_CONSTANT),
    Buffer.from(ticker),
  ];
  const [poolPDA] = anchor.web3.PublicKey.findProgramAddressSync(
    seeds,
    program.programId
  );
  return poolPDA;
};

export const getPoolLPMint = async (ticker: string) => {
  const POOL_LP_MINT_SEED_CONSTANT = 'pool_lp_mint';
  const seeds = [
    Buffer.from(POOL_LP_MINT_SEED_CONSTANT),
    Buffer.from(ticker),
  ];
  const [poolLPMint] = anchor.web3.PublicKey.findProgramAddressSync(
    seeds,
    program.programId
  );
  return poolLPMint;
};
