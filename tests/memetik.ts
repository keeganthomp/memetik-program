import * as anchor from '@coral-xyz/anchor';
import { assert } from 'chai';
import {
  fundSol,
  getMintPDA,
  getMetadataPDA,
  getPoolPDA,
  getSOLBalance,
  getSPLBalance,
  getSol,
  logTxnInfo,
} from './utils';
import { Memetik } from '../target/types/memetik';

// Configure the client to use the local cluster.
anchor.setProvider(anchor.AnchorProvider.env());
const program = anchor.workspace.Memetik as anchor.Program<Memetik>;

const userA = anchor.web3.Keypair.generate();
const userB = anchor.web3.Keypair.generate();
const userC = anchor.web3.Keypair.generate();
const users = [userA, userB, userC];

const tokens = [
  {
    symbol: 'MEME',
    name: 'MEME',
    uri: 'https://arweave.net/123',
  },
  {
    symbol: 'MEME2',
    name: 'MEME2',
    uri: 'https://arweave.net/123',
  },
];

describe('memetik', () => {
  const createdPools: any[] = [];

  before(async () => {
    for (const user of users) {
      await fundSol(user.publicKey, 1000);
    }
  });

  it('Can launch token', async () => {
    const creator = userA;
    const tokenInfo = tokens[0];
    const mint = getMintPDA(tokenInfo.symbol);
    const metadata = getMetadataPDA(mint);
    try {
      const creatorSolBalBefore = await getSOLBalance(creator.publicKey);
      await program.methods
        .initializePool(tokenInfo.symbol, tokenInfo.name, tokenInfo.uri)
        .accounts({
          signer: creator.publicKey,
          metadata,
        })
        .signers([creator])
        .rpc();
      const creatorSolBalAfter = await getSOLBalance(creator.publicKey);
      const costToCreate = creatorSolBalBefore - creatorSolBalAfter;
      console.log('Cost to create pool (SOL):', getSol(costToCreate));
      const poolPDA = getPoolPDA(tokenInfo.symbol);
      const pool = await program.account.bondingPool.fetch(poolPDA);
      createdPools.push(pool);
    } catch (err) {
      console.log('err', err);
      assert.fail();
    }
  });

  it('Can buy on bonding curve', async () => {
    const buyer = userB;
    const pool = createdPools[0];
    const BUY_AMOUNT = 100_000_000;
    try {
      const buyerSolBalBefore = await getSOLBalance(buyer.publicKey);
      const buyerTokenAccount = await anchor.utils.token.associatedAddress(
        {
          mint: getMintPDA(pool.ticker),
          owner: buyer.publicKey,
        }
      );
      const buyerTokenBalBefore = await getSPLBalance(buyerTokenAccount);
      await program.methods
        .buy(pool.ticker, new anchor.BN(BUY_AMOUNT))
        .accounts({
          buyer: buyer.publicKey,
        })
        .signers([buyer])
        .rpc();
      const buyerTokenBalAfter = await getSPLBalance(buyerTokenAccount);
      const buyerSolBalAfter = await getSOLBalance(buyer.publicKey);
      assert.ok(buyerSolBalAfter < buyerSolBalBefore);
      assert.ok(buyerTokenBalAfter > buyerTokenBalBefore);
    } catch (err) {
      console.log('err buying', err);
      assert.fail();
    }
  });

  it('Can sell on bonding curve', async () => {
    const seller = userB;
    const pool = createdPools[0];
    const SELL_AMOUNT = 100;
    try {
      const sellerSolBalBefore = await getSOLBalance(seller.publicKey);
      const sellerTokenAccount =
        await anchor.utils.token.associatedAddress({
          mint: getMintPDA(pool.ticker),
          owner: seller.publicKey,
        });
      const sellerTokenBalBefore = await getSPLBalance(sellerTokenAccount);
      await program.methods
        .sell(pool.ticker, new anchor.BN(SELL_AMOUNT))
        .accounts({
          seller: seller.publicKey,
        })
        .signers([seller])
        .rpc();
      const sellerTokenBalAfter = await getSPLBalance(sellerTokenAccount);
      const sellerSolBalAfter = await getSOLBalance(seller.publicKey);
      assert.ok(sellerSolBalAfter >= sellerSolBalBefore);
      assert.ok(sellerTokenBalAfter <= sellerTokenBalBefore);
    } catch (err) {
      console.log('err buying', err);
      assert.fail();
    }
  });

  it('Can not close pool early', async () => {
    const creator = userA;
    const pool = createdPools[0];
    try {
      await program.methods
        .close(pool.ticker)
        .accounts({
          signer: creator.publicKey,
        })
        .signers([creator])
        .rpc();
      assert.fail();
    } catch (err) {
      assert(err?.error?.errorCode?.code === 'PoolCannotBeClosed');
    }
  });

  it('Can add liquidity', async () => {
    const liquidityProvider = userB;
    const pool = createdPools[0];
    const SOL_LIQ_AMOUNT = 10000;
    const TOKEN_LIQ_AMOUNT = 10000;
    try {
      const lpSolBalBefore = await getSOLBalance(
        liquidityProvider.publicKey
      );
      const txn = await program.methods
        .addLiquidity(
          pool.ticker,
          new anchor.BN(SOL_LIQ_AMOUNT),
          new anchor.BN(TOKEN_LIQ_AMOUNT)
        )
        .accounts({
          user: liquidityProvider.publicKey,
        })
        .signers([liquidityProvider])
        .rpc();
      await logTxnInfo(txn);
      const lpSolBalAfter = await getSOLBalance(
        liquidityProvider.publicKey
      );
      assert.ok(lpSolBalAfter < lpSolBalBefore);
    } catch (err) {
      console.log('err adding liquidity', err);
      assert.fail();
    }
  });

  it('Can swap SOL for token', async () => {
    const swapper = userB;
    const pool = createdPools[0];
    const SOL_SWAP_AMOUNT = 7;
    try {
      const swapperSolBalBefore = await getSOLBalance(swapper.publicKey);
      const swapperTokenAccount =
        await anchor.utils.token.associatedAddress({
          mint: getMintPDA(pool.ticker),
          owner: swapper.publicKey,
        });
      const swapperTokenBalBefore = await getSPLBalance(
        swapperTokenAccount
      );
      const txn = await program.methods
        .swap(pool.ticker, new anchor.BN(SOL_SWAP_AMOUNT), true)
        .accounts({
          user: swapper.publicKey,
        })
        .signers([swapper])
        .rpc();
      await logTxnInfo(txn);
      const swapperSolBalAfter = await getSOLBalance(swapper.publicKey);
      const swapperTokenBalAfter = await getSPLBalance(
        swapperTokenAccount
      );
      assert.ok(swapperSolBalAfter < swapperSolBalBefore);
      assert.ok(swapperTokenBalAfter > swapperTokenBalBefore);
    } catch (err) {
      console.log('Err swapping', err);
      assert.fail();
    }
  });

  it('Can swap token for SOL', async () => {
    const swapper = userB;
    const pool = createdPools[0];
    const TOKEN_SWAP_AMOUNT = 7;
    try {
      const swapperSolBalBefore = await getSOLBalance(swapper.publicKey);
      const swapperTokenAccount =
        await anchor.utils.token.associatedAddress({
          mint: getMintPDA(pool.ticker),
          owner: swapper.publicKey,
        });
      const swapperTokenBalBefore = await getSPLBalance(
        swapperTokenAccount
      );
      const txn = await program.methods
        .swap(pool.ticker, new anchor.BN(TOKEN_SWAP_AMOUNT), false)
        .accounts({
          user: swapper.publicKey,
        })
        .signers([swapper])
        .rpc();
      await logTxnInfo(txn);
      const swapperSolBalAfter = await getSOLBalance(swapper.publicKey);
      const swapperTokenBalAfter = await getSPLBalance(
        swapperTokenAccount
      );
      assert.ok(swapperSolBalAfter > swapperSolBalBefore);
      assert.ok(swapperTokenBalAfter < swapperTokenBalBefore);
    } catch (err) {
      console.log('Err swapping', err);
      assert.fail();
    }
  });
});
