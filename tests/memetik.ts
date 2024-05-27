import * as anchor from '@coral-xyz/anchor';
import { Memetik } from '../target/types/memetik';
import { assert } from 'chai';
import { getLogs } from '@solana-developers/helpers';
import { LAMPORTS_PER_SOL } from '@solana/web3.js';

const getLamports = (amount: number) => {
  return amount * LAMPORTS_PER_SOL;
};

const getSol = (lamports: number) => {
  return lamports / LAMPORTS_PER_SOL;
};

const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey(
  'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s'
);

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

const getSOLBalance = async (account) => {
  const balance = await provider.connection.getBalance(account);
  return balance;
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

export const logTxnInfo = async (
  txn: anchor.web3.TransactionSignature
) => {
  await waitForTxnConfrimation(txn);
  const logs = await getLogs(provider.connection, txn);
  console.log('Transaction logs:', logs);
};

const getGlobalStatePda = () => {
  const GLOBAL_SEED_CONSTANT = 'global-state';
  const [globalStatePda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from(GLOBAL_SEED_CONSTANT)],
    program.programId
  );
  return globalStatePda;
};

const getMintPDA = (poolId: number | anchor.BN) => {
  const MINT_SEED_CONSTANT = 'mint';
  let poolIdNum = typeof poolId === 'number' ? poolId : poolId.toNumber();
  const seeds = [
    Buffer.from(MINT_SEED_CONSTANT),
    Buffer.from(
      new Uint8Array(new BigUint64Array([BigInt(poolIdNum)]).buffer)
    ),
  ];
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

const getPoolPDA = (poolId: number | anchor.BN) => {
  const POOL_SEED_CONSTANT = 'pool';
  let poolIdNum = typeof poolId === 'number' ? poolId : poolId.toNumber();
  const seeds = [
    Buffer.from(POOL_SEED_CONSTANT),
    Buffer.from(
      new Uint8Array(new BigUint64Array([BigInt(poolIdNum)]).buffer)
    ),
  ];
  const [poolPDA] = anchor.web3.PublicKey.findProgramAddressSync(
    seeds,
    program.programId
  );
  return poolPDA;
};

const getNextPoolId = async () => {
  const globalStatePda = getGlobalStatePda();
  try {
    const globalState = await program.account.globalState.fetch(
      globalStatePda
    );
    return globalState.poolsCreated.toNumber() + 1;
  } catch (err) {
    const accountDoesNotExist = err?.message?.includes(
      'Account does not exist'
    );
    if (accountDoesNotExist) return 1;
    throw err;
  }
};

const buyTokens = async (buyer: any, pool: any, amount: number) => {
  const poolPDA = getPoolPDA(pool.id.toNumber());
  const poolFromProgram = await program.account.pool.fetch(poolPDA);
  const buyerTokenAccount = await anchor.utils.token.associatedAddress({
    mint: getMintPDA(pool.id.toNumber()),
    owner: buyer.publicKey,
  });
  const priceInSol = getSol(poolFromProgram.tokPrice.toNumber());
  console.log(
    `Buyer buying ${amount} tokens for ${priceInSol} SOL per token`
  );
  const txn = await program.methods
    .buy(pool.id, new anchor.BN(amount))
    .accounts({
      buyer: buyer.publicKey,
      buyerTokenAccount,
    })
    .signers([buyer])
    .rpc();
  await logTxnInfo(txn);
  return txn;
};

const sellTokens = async (seller: any, pool: any, amount: number) => {
  const poolPDA = getPoolPDA(pool.id.toNumber());
  const poolFromPogram = await program.account.pool.fetch(poolPDA);
  const sellerTokenAccount = await anchor.utils.token.associatedAddress({
    mint: getMintPDA(pool.id.toNumber()),
    owner: seller.publicKey,
  });
  const priceInSol = getSol(poolFromPogram.tokPrice.toNumber());
  console.log(
    `Seller selling ${amount} tokens at ${priceInSol} SOL per token`
  );
  const txn = await program.methods
    .sell(pool.id, new anchor.BN(amount))
    .accounts({
      seller: seller.publicKey,
      sellerTokenAccount,
    })
    .signers([seller])
    .rpc();
  await logTxnInfo(txn);
  return txn;
};

const userA = anchor.web3.Keypair.generate();
const userB = anchor.web3.Keypair.generate();
const userC = anchor.web3.Keypair.generate();
const users = [userA, userB, userC];

describe('memetik', () => {
  const createdPools: any[] = [];

  before(async () => {
    for (const user of users) {
      await fundSol(user.publicKey);
    }
  });

  it('Can launch token', async () => {
    const tok = {
      name: 'wee',
      symbol: 'WEE',
      uri: '',
    };
    try {
      const creator = userA;
      const poolId = await getNextPoolId();
      const mint = getMintPDA(poolId);
      const metadata = getMetadataPDA(mint);
      const txn = await program.methods
        .initialize(new anchor.BN(poolId), tok)
        .accounts({
          signer: creator.publicKey,
          metadata,
        })
        .signers([creator])
        .rpc();
      await waitForTxnConfrimation(txn);
      const pool = await program.account.pool.fetch(getPoolPDA(poolId));
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

  it('Can launch another token', async () => {
    const tok = {
      name: 'balls',
      symbol: 'BALL',
      uri: '',
    };
    const creator = userA;
    const poolId = await getNextPoolId();
    const mint = getMintPDA(poolId);
    const metadata = getMetadataPDA(mint);
    try {
      const txn = await program.methods
        .initialize(new anchor.BN(poolId), tok)
        .accounts({
          signer: creator.publicKey,
          metadata,
        })
        .signers([creator])
        .rpc();
      await waitForTxnConfrimation(txn);
      const pool = await program.account.pool.fetch(getPoolPDA(poolId));
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

  it('Can buy tokens', async () => {
    const buyer = userB;
    const amount = 1;
    const firstPool = createdPools[0];
    try {
      const buyerTokenAccount = await anchor.utils.token.associatedAddress(
        {
          mint: getMintPDA(firstPool.id.toNumber()),
          owner: buyer.publicKey,
        }
      );
      const solBalBefore = await getSOLBalance(buyer.publicKey);
      const tokBalBefore = await getSPLBalance(buyerTokenAccount);
      await buyTokens(buyer, firstPool, amount);
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
    const poolPDA = getPoolPDA(pool.id.toNumber());
    try {
      const buyerTokenAccount = await anchor.utils.token.associatedAddress(
        {
          mint: getMintPDA(pool.id.toNumber()),
          owner: buyer.publicKey,
        }
      );
      const totalAmountToBy = getLamports(60);
      const batchAmount = totalAmountToBy / 12;
      let amountPurchased = 0;
      while (amountPurchased < totalAmountToBy) {
        const pool = await program.account.pool.fetch(poolPDA);
        let tokPriceBefore = pool.tokPrice;
        let solBalBefore = await getSOLBalance(buyer.publicKey);
        let tokBalBefore = await getSPLBalance(buyerTokenAccount);
        await buyTokens(buyer, pool, batchAmount);
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
    const poolPDA = getPoolPDA(pool.id.toNumber());
    try {
      const poolBefore = await program.account.pool.fetch(poolPDA);
      const sellerTokenAccount =
        await anchor.utils.token.associatedAddress({
          mint: getMintPDA(pool.id.toNumber()),
          owner: seller.publicKey,
        });
      const sellerSolBalBefore = await getSOLBalance(seller.publicKey);
      const sellerTokBalBefore = await getSPLBalance(sellerTokenAccount);
      const priceBefore = poolBefore.tokPrice;
      await sellTokens(seller, pool, sellerTokBalBefore);
      const sellerSolBalAfter = await getSOLBalance(seller.publicKey);
      const sellerTokBalAfter = await getSPLBalance(sellerTokenAccount);
      const poolAfter = await program.account.pool.fetch(poolPDA);
      assert.ok(sellerSolBalAfter >= sellerSolBalBefore);
      assert.ok(sellerTokBalAfter < sellerTokBalBefore);
      console.log('price before sell', priceBefore);
      console.log('price after sell', poolAfter.tokPrice);
      assert.ok(poolAfter.tokPrice.toNumber() <= priceBefore.toNumber());
    } catch (err) {
      console.log('Can sell err', err);
      assert.fail('Transaction failed');
    }
  });
  it('Token price decresses when supply decreases', async () => {
    const seller = userC;
    const pool = createdPools[0];
    const poolPDA = getPoolPDA(pool.id.toNumber());
    try {
      const sellerTokenAccount =
        await anchor.utils.token.associatedAddress({
          mint: getMintPDA(pool.id.toNumber()),
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
        await sellTokens(seller, pool, batchAmount);
        const poolAfter = await program.account.pool.fetch(poolPDA);
        const solBalanceAfter = await getSOLBalance(seller.publicKey);
        const tokenBalanceAfter = await getSPLBalance(sellerTokenAccount);
        console.log('price before sell', tokPriceBefore);
        console.log('price after sell', poolAfter.tokPrice);
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
});
