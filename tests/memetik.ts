import * as anchor from '@coral-xyz/anchor';
import { Memetik } from '../target/types/memetik';
import { assert } from 'chai';
import { getLogs } from '@solana-developers/helpers';
import { LAMPORTS_PER_SOL } from '@solana/web3.js';

type PoolFromProgram = {
  ticker: string;
  tokPrice: anchor.BN;
  mint: anchor.web3.PublicKey;
  creator: anchor.web3.PublicKey;
  maturityTime: anchor.BN;
};

const getLamports = (amount: number) => {
  return amount * LAMPORTS_PER_SOL;
};
const getSol = (lamports: number) => {
  return lamports / LAMPORTS_PER_SOL;
};

const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey(
  'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s'
);

// list of tokens to be created
const tokens = [
  {
    name: 'wee',
    symbol: 'WEE',
    uri: '',
  },
  {
    name: 'balls',
    symbol: 'WOO',
    uri: '',
  },
];

// Configure the client to use the local cluster.
anchor.setProvider(anchor.AnchorProvider.env());
const program = anchor.workspace.Memetik as anchor.Program<Memetik>;

const provider = anchor.getProvider();

const getSPLBalance = async (tokenAccount) => {
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

const getSOLBalance = async (account) => {
  const balance = await provider.connection.getBalance(account);
  return balance;
};

const logTxnInfo = async (txn: anchor.web3.TransactionSignature) => {
  await waitForTxnConfrimation(txn);
  const logs = await getLogs(provider.connection, txn);
  console.log('Transaction logs:', logs);
};

const waitUntilTime = async (targetTimestamp) => {
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

const fundSol = async (
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

const getMintPDA = (ticker: string) => {
  const MINT_SEED_CONSTANT = 'mint';
  const seeds = [Buffer.from(MINT_SEED_CONSTANT), Buffer.from(ticker)];
  const [mintPDA] = anchor.web3.PublicKey.findProgramAddressSync(
    seeds,
    program.programId
  );
  return mintPDA;
};
const getMetadataPDA = (mint: anchor.web3.PublicKey) => {
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
const getPoolPDA = (ticker: string) => {
  const POOL_SEED_CONSTANT = 'pool';
  const seeds = [Buffer.from(POOL_SEED_CONSTANT), Buffer.from(ticker)];
  const [poolPDA] = anchor.web3.PublicKey.findProgramAddressSync(
    seeds,
    program.programId
  );
  return poolPDA;
};
const getEscrowPDA = (ticker: string) => {
  const ESCROW_SEED_CONSTANT = 'pool-escrow';
  const seeds = [Buffer.from(ESCROW_SEED_CONSTANT), Buffer.from(ticker)];
  const [escrowPDA] = anchor.web3.PublicKey.findProgramAddressSync(
    seeds,
    program.programId
  );
  return escrowPDA;
};

const buyTokens = async (buyer: any, ticker: string, amount: number) => {
  const poolPDA = getPoolPDA(ticker);
  const poolFromProgram = await program.account.pool.fetch(poolPDA);
  const buyerTokenAccount = await anchor.utils.token.associatedAddress({
    mint: getMintPDA(ticker),
    owner: buyer.publicKey,
  });
  const priceInSol = getSol(poolFromProgram.tokPrice.toNumber());
  console.log(
    `Buyer buying ${amount} tokens for ${priceInSol} SOL per token`
  );
  const txn = await program.methods
    .buy(ticker, new anchor.BN(amount))
    .accounts({
      buyer: buyer.publicKey,
      buyerTokenAccount,
    })
    .signers([buyer])
    .rpc();
  return txn;
};

const sellTokens = async (seller: any, ticker: string, amount: number) => {
  const poolPDA = getPoolPDA(ticker);
  const poolFromPogram = await program.account.pool.fetch(poolPDA);
  const sellerTokenAccount = await anchor.utils.token.associatedAddress({
    mint: getMintPDA(ticker),
    owner: seller.publicKey,
  });
  const priceInSol = getSol(poolFromPogram.tokPrice.toNumber());
  console.log(
    `Seller selling ${amount} tokens at ${priceInSol} SOL per token`
  );
  const txn = await program.methods
    .sell(ticker, new anchor.BN(amount))
    .accounts({
      seller: seller.publicKey,
      sellerTokenAccount,
    })
    .signers([seller])
    .rpc();
  return txn;
};

const userA = anchor.web3.Keypair.generate();
const userB = anchor.web3.Keypair.generate();
const userC = anchor.web3.Keypair.generate();
const users = [userA, userB, userC];

describe('memetik', () => {
  const createdPools: PoolFromProgram[] = [];

  before(async () => {
    for (const user of users) {
      await fundSol(user.publicKey);
    }
  });

  it('Can launch token', async () => {
    const token = tokens[0];
    try {
      const creator = userA;
      const mint = getMintPDA(token.symbol);
      const metadata = getMetadataPDA(mint);
      const txn = await program.methods
        .initialize(token)
        .accounts({
          signer: creator.publicKey,
          metadata,
        })
        .signers([creator])
        .rpc();
      await waitForTxnConfrimation(txn);
      const pool = await program.account.pool.fetch(
        getPoolPDA(token.symbol)
      );
      const escrowPDA = getEscrowPDA(token.symbol);
      const escrowSolBalance = await getSOLBalance(escrowPDA);
      const escrowAcc = await program.account.poolEscrow.fetch(escrowPDA);
      const creatorTokenAccount =
        await anchor.utils.token.associatedAddress({
          mint: mint,
          owner: creator.publicKey,
        });
      const creatorTokenBalance = await getSPLBalance(creatorTokenAccount);
      assert.ok(pool);
      assert.ok(creatorTokenBalance === 0);
      assert.ok(escrowSolBalance > 0);
      assert.ok(escrowSolBalance >= escrowAcc.balance.toNumber());
      createdPools.push(pool);
      console.log('Token Price at creation:', pool.tokPrice);
    } catch (err) {
      console.log('Create token', err);
      assert.fail('Transaction failed');
    }
  });

  it("Can fetch pool", async () => {
    const pool = createdPools[0];
    const poolFromProgram = await program.methods.getPool(pool.ticker).view();
    assert.ok(poolFromProgram.ticker === pool.ticker);
  })

  it('Can launch another token', async () => {
    const token = tokens[1];
    const creator = userA;
    const mint = getMintPDA(token.symbol);
    const metadata = getMetadataPDA(mint);
    try {
      const txn = await program.methods
        .initialize(token)
        .accounts({
          signer: creator.publicKey,
          metadata,
        })
        .signers([creator])
        .rpc();
      await waitForTxnConfrimation(txn);
      const pool = await program.account.pool.fetch(
        getPoolPDA(token.symbol)
      );
      const creatorTokenAccount =
        await anchor.utils.token.associatedAddress({
          mint: mint,
          owner: creator.publicKey,
        });
      const creatorTokenBalance = await getSPLBalance(creatorTokenAccount);
      assert.ok(pool);
      assert.ok(creatorTokenBalance === 0);
      createdPools.push(pool);
      console.log('Token Price at creation:', pool.tokPrice);
    } catch (err) {
      console.log('Create token', err);
      assert.fail('Transaction failed');
    }
  });

  it('can NOT close pool before maturity', async () => {
    const creator = userA;
    const pool = createdPools[0];
    try {
      await program.methods
        .close(pool.ticker)
        .accounts({
          creator: creator.publicKey,
        })
        .signers([creator])
        .rpc();
      assert.fail('Transaction failed');
    } catch (err) {
      assert.ok(err?.error?.errorCode?.code === 'PoolNotMatured');
    }
  });

  it('Can buy tokens', async () => {
    const buyer = userB;
    const amount = 1;
    const pool = createdPools[0];
    try {
      const buyerTokenAccount = await anchor.utils.token.associatedAddress(
        {
          mint: getMintPDA(pool.ticker),
          owner: buyer.publicKey,
        }
      );
      const solBalBefore = await getSOLBalance(buyer.publicKey);
      const tokBalBefore = await getSPLBalance(buyerTokenAccount);
      await buyTokens(buyer, pool.ticker, amount);
      const solBalAfter = await getSOLBalance(buyer.publicKey);
      const tokBalAfter = await getSPLBalance(buyerTokenAccount);
      assert.ok(solBalAfter < solBalBefore);
      assert.ok(tokBalAfter > tokBalBefore);
    } catch (err) {
      console.log('Can buy err', err);
      assert.fail('Transaction failed');
    }
  });
  it('Token price increases with demand', async () => {
    const buyer = userC;
    const pool = createdPools[0];
    const poolPDA = getPoolPDA(pool.ticker);
    try {
      const buyerTokenAccount = await anchor.utils.token.associatedAddress(
        {
          mint: getMintPDA(pool.ticker),
          owner: buyer.publicKey,
        }
      );
      const totalAmountToBy = getLamports(250);
      const batchAmount = totalAmountToBy / 12;
      let amountPurchased = 0;
      while (amountPurchased < totalAmountToBy) {
        const pool = await program.account.pool.fetch(poolPDA);
        let tokPriceBefore = pool.tokPrice;
        let solBalBefore = await getSOLBalance(buyer.publicKey);
        let tokBalBefore = await getSPLBalance(buyerTokenAccount);
        await buyTokens(buyer, pool.ticker, batchAmount);
        const poolAfter = await program.account.pool.fetch(poolPDA);
        const solBalanceAfter = await getSOLBalance(buyer.publicKey);
        const tokenBalanceAfter = await getSPLBalance(buyerTokenAccount);
        console.log('solBalBefore', solBalBefore);
        console.log('solBalanceAfter', solBalanceAfter);
        console.log('tokBalBefore', tokBalBefore);
        console.log('tokenBalanceAfter', tokenBalanceAfter);
        console.log('New Token price:', poolAfter.tokPrice.toNumber());
        assert.ok(solBalanceAfter < solBalBefore);
        assert.ok(tokenBalanceAfter > tokBalBefore);
        assert.ok(
          poolAfter.tokPrice.toNumber() >= tokPriceBefore.toNumber()
        );
        tokPriceBefore = poolAfter.tokPrice;
        solBalBefore = solBalanceAfter;
        amountPurchased += batchAmount;
      }
    } catch (err) {
      console.log('Can buy demand error', err);
      assert.fail('Transaction failed');
    }
  });
  it('Can sell tokens', async () => {
    const seller = userB;
    const pool = createdPools[0];
    const poolPDA = getPoolPDA(pool.ticker);
    try {
      const poolBefore = await program.account.pool.fetch(poolPDA);
      const sellerTokenAccount =
        await anchor.utils.token.associatedAddress({
          mint: getMintPDA(pool.ticker),
          owner: seller.publicKey,
        });
      const sellerSolBalBefore = await getSOLBalance(seller.publicKey);
      const sellerTokBalBefore = await getSPLBalance(sellerTokenAccount);
      const priceBefore = poolBefore.tokPrice;
      await sellTokens(seller, pool.ticker, sellerTokBalBefore);
      const sellerSolBalAfter = await getSOLBalance(seller.publicKey);
      const sellerTokBalAfter = await getSPLBalance(sellerTokenAccount);
      const poolAfter = await program.account.pool.fetch(poolPDA);
      assert.ok(sellerSolBalAfter >= sellerSolBalBefore);
      assert.ok(sellerTokBalAfter < sellerTokBalBefore);
      assert.ok(poolAfter.tokPrice.toNumber() <= priceBefore.toNumber());
    } catch (err) {
      console.log('Can sell err', err);
      assert.fail('Transaction failed');
    }
  });
  it('Token price decresses when supply decreases', async () => {
    const seller = userC;
    const pool = createdPools[0];
    const poolPDA = getPoolPDA(pool.ticker);
    try {
      const sellerTokenAccount =
        await anchor.utils.token.associatedAddress({
          mint: getMintPDA(pool.ticker),
          owner: seller.publicKey,
        });
      const totalAmountToSell = await getSPLBalance(sellerTokenAccount);
      const batchAmount = totalAmountToSell / 5;
      let amountSold = 0;
      while (amountSold <= totalAmountToSell) {
        const pool = await program.account.pool.fetch(poolPDA);
        let tokPriceBefore = pool.tokPrice;
        let solBalBefore = await getSOLBalance(seller.publicKey);
        let tokBalBefore = await getSPLBalance(sellerTokenAccount);
        if (tokBalBefore - 1 < batchAmount) break;
        await sellTokens(seller, pool.ticker, batchAmount);
        const poolAfter = await program.account.pool.fetch(poolPDA);
        const solBalanceAfter = await getSOLBalance(seller.publicKey);
        const tokenBalanceAfter = await getSPLBalance(sellerTokenAccount);
        assert.ok(tokenBalanceAfter < tokBalBefore);
        assert.ok(solBalanceAfter > solBalBefore);
        assert.ok(
          poolAfter.tokPrice.toNumber() <= tokPriceBefore.toNumber()
        );
        solBalBefore = solBalanceAfter;
        tokPriceBefore = poolAfter.tokPrice;
        amountSold += batchAmount;
        tokBalBefore = tokenBalanceAfter;
        console.log('tok bal after', tokenBalanceAfter);
        console.log('sol bal after', solBalanceAfter);
      }
    } catch (err) {
      console.log('Selling demand decreases', err);
      assert.fail('Transaction failed');
    }
  });

  it('bad actor can NOT close pool', async () => {
    const badActor = userB;
    const pool = createdPools[0];
    const maturityTimeStampMs = pool.maturityTime.toNumber() * 1000;
    try {
      await waitUntilTime(maturityTimeStampMs);
      await program.methods
        .close(pool.ticker)
        .accounts({
          creator: badActor.publicKey,
        })
        .signers([badActor])
        .rpc();
      assert.fail('Transaction failed');
    } catch (err) {
      assert.ok(err?.error?.errorCode?.code === 'NotPoolCreator');
    }
  });

  it('creator can close pool after maturity date', async () => {
    const creator = userA;
    const pool = createdPools[0];
    const maturityTimeStampMs = pool.maturityTime.toNumber() * 1000;
    const creatorBalBefore = await getSOLBalance(creator.publicKey);
    try {
      await waitUntilTime(maturityTimeStampMs);
      await program.methods
        .close(pool.ticker)
        .accounts({
          creator: creator.publicKey,
        })
        .signers([creator])
        .rpc();
      const creatorBalAfter = await getSOLBalance(creator.publicKey);
      assert.ok(creatorBalAfter > creatorBalBefore);
    } catch (err) {
      console.log('Close pool error', err);
      assert.fail('Transaction failed');
    }
  });
});
