import * as anchor from '@coral-xyz/anchor';
import { assert } from 'chai';
import {
  fundSol,
  getMintPDA,
  getMetadataPDA,
  getPoolPDA,
  getPoolLPMint,
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

const buyTokensOnCurve = async (
  ticker: string,
  buyer: anchor.web3.Keypair,
  amount: number
) => {
  const buyerSolBalBefore = await getSOLBalance(buyer.publicKey);
  const buyerTokenAccount = await anchor.utils.token.associatedAddress({
    mint: getMintPDA(ticker),
    owner: buyer.publicKey,
  });
  const buyerTokenBalBefore = await getSPLBalance(buyerTokenAccount);
  const txn = await program.methods
    .buy(ticker, new anchor.BN(amount))
    .accounts({
      buyer: buyer.publicKey,
    })
    .signers([buyer])
    .rpc();
  const buyerTokenBalAfter = await getSPLBalance(buyerTokenAccount);
  const buyerSolBalAfter = await getSOLBalance(buyer.publicKey);
  assert.ok(buyerSolBalAfter < buyerSolBalBefore);
  assert.ok(buyerTokenBalAfter > buyerTokenBalBefore);
  return txn;
};

const sellTokensOnCurve = async (
  ticker: string,
  seller: anchor.web3.Keypair,
  amount: number
) => {
  const sellerSolBalBefore = await getSOLBalance(seller.publicKey);
  const sellerTokenAccount = await anchor.utils.token.associatedAddress({
    mint: getMintPDA(ticker),
    owner: seller.publicKey,
  });
  const sellerTokenBalBefore = await getSPLBalance(sellerTokenAccount);
  const txn = await program.methods
    .sell(ticker, new anchor.BN(amount))
    .accounts({
      seller: seller.publicKey,
    })
    .signers([seller])
    .rpc();
  const sellerTokenBalAfter = await getSPLBalance(sellerTokenAccount);
  const sellerSolBalAfter = await getSOLBalance(seller.publicKey);
  assert.ok(sellerSolBalAfter >= sellerSolBalBefore);
  assert.ok(sellerTokenBalAfter < sellerTokenBalBefore);
  return txn;
};

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
      console.log(
        'Cost to create pool (SOL):',
        getSol(costToCreate).toFixed(3)
      );
      const poolPDA = getPoolPDA(tokenInfo.symbol);
      const pool = await program.account.bondingPool.fetch(poolPDA);
      createdPools.push(pool);
    } catch (err) {
      console.log('err', err);
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

  it('Can buy on bonding curve', async () => {
    const buyer = userB;
    const pool = createdPools[0];
    const BUY_AMOUNT = 100;
    try {
      await buyTokensOnCurve(pool.ticker, buyer, BUY_AMOUNT);
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
      await sellTokensOnCurve(pool.ticker, seller, SELL_AMOUNT);
    } catch (err) {
      console.log('Err selling', err);
      assert.fail();
    }
  });

  it('Token price increases after buying on curve', async () => {
    const buyer = userB;
    const pool = createdPools[0];
    const TOTAL_AMOUNT_TO_BUY = 10_000;
    const PURCHASE_BATCH_SIZE = 2_000;
    try {
      let amountPurchased = 0;
      while (amountPurchased < TOTAL_AMOUNT_TO_BUY) {
        const amountToBuy = Math.min(
          TOTAL_AMOUNT_TO_BUY,
          PURCHASE_BATCH_SIZE
        );
        console.log(`Buying ${amountToBuy} tokens on curve...`);
        const poolBeforePurchase = await program.account.bondingPool.fetch(
          getPoolPDA(pool.ticker)
        );
        const txn = await buyTokensOnCurve(
          pool.ticker,
          buyer,
          amountToBuy
        );
        const poolAfterPurchase = await program.account.bondingPool.fetch(
          getPoolPDA(pool.ticker)
        );
        const tokenPriceBefore =
          poolBeforePurchase.lastTokenPrice.toNumber();
        const tokenPriceAfter =
          poolAfterPurchase.lastTokenPrice.toNumber();
        assert.ok(tokenPriceAfter >= tokenPriceBefore);
        amountPurchased += amountToBuy;
      }
    } catch (err) {
      console.log('err buying', err);
      assert.fail();
    }
  });

  it('Token price decreases after selling on curve', async () => {
    const seller = userB;
    const pool = createdPools[0];
    const TOTAL_AMOUNT_TO_SELL = 5_000;
    const SELL_BATCH_SIZE = 1_200;
    try {
      let amountSold = 0;
      while (amountSold < TOTAL_AMOUNT_TO_SELL) {
        const amountToSell = Math.min(
          TOTAL_AMOUNT_TO_SELL,
          SELL_BATCH_SIZE
        );
        console.log(`Selling ${amountToSell} tokens on curve...`);
        const poolBeforeSell = await program.account.bondingPool.fetch(
          getPoolPDA(pool.ticker)
        );
        const txn = await sellTokensOnCurve(
          pool.ticker,
          seller,
          amountToSell
        );
        const poolAfterSell = await program.account.bondingPool.fetch(
          getPoolPDA(pool.ticker)
        );
        const tokenPriceBefore = poolBeforeSell.lastTokenPrice.toNumber();
        const tokenPriceAfter = poolAfterSell.lastTokenPrice.toNumber();
        assert.ok(tokenPriceAfter <= tokenPriceBefore);
        amountSold += amountToSell;
      }
    } catch (err) {
      console.log('err buying', err);
      assert.fail();
    }
  });

  it('Can add liquidity', async () => {
    const liquidityProvider = userB;
    const pool = createdPools[0];
    try {
      const poolLPMint = await getPoolLPMint(pool.ticker);
      const lpSolBalBefore = await getSOLBalance(
        liquidityProvider.publicKey
      );
      const lpTokenAccount = await anchor.utils.token.associatedAddress({
        mint: poolLPMint,
        owner: liquidityProvider.publicKey,
      });
      const lpTokenBalBefore = await getSPLBalance(lpTokenAccount);
      await program.methods
        .addLiquidity(
          pool.ticker,
          new anchor.BN(100),
          new anchor.BN(100)
        )
        .accounts({
          user: liquidityProvider.publicKey,
        })
        .signers([liquidityProvider])
        .rpc();
      const lpSolBalAfter = await getSOLBalance(
        liquidityProvider.publicKey
      );
      const lpTokenBalAfter = await getSPLBalance(lpTokenAccount);
      assert.ok(lpSolBalAfter < lpSolBalBefore);
      assert.ok(lpTokenBalAfter > lpTokenBalBefore);
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
      await program.methods
        .swap(pool.ticker, new anchor.BN(SOL_SWAP_AMOUNT), true)
        .accounts({
          user: swapper.publicKey,
        })
        .signers([swapper])
        .rpc();
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
      await program.methods
        .swap(pool.ticker, new anchor.BN(TOKEN_SWAP_AMOUNT), false)
        .accounts({
          user: swapper.publicKey,
        })
        .signers([swapper])
        .rpc();
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

  it('Can remove liquidity', async () => {
    const liquidityProvider = userB;
    const pool = createdPools[0];
    const LIQ_AMOUNT = 5;
    try {
      const poolLPMint = await getPoolLPMint(pool.ticker);
      const lpSolBalBefore = await getSOLBalance(
        liquidityProvider.publicKey
      );
      const lpTokenAccount = await anchor.utils.token.associatedAddress({
        mint: poolLPMint,
        owner: liquidityProvider.publicKey,
      });
      const lpTokenBalBefore = await getSPLBalance(lpTokenAccount);
      await program.methods
        .removeLiquidity(pool.ticker, new anchor.BN(LIQ_AMOUNT))
        .accounts({
          user: liquidityProvider.publicKey,
        })
        .signers([liquidityProvider])
        .rpc();
      const lpSolBalAfter = await getSOLBalance(
        liquidityProvider.publicKey
      );
      const lpTokenBalAfter = await getSPLBalance(lpTokenAccount);
      assert.ok(lpSolBalAfter > lpSolBalBefore);
      assert.ok(lpTokenBalAfter < lpTokenBalBefore);
    } catch (err) {
      console.log('err removing liquidity', err);
      assert.fail();
    }
  });
});
