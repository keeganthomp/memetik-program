import * as anchor from '@coral-xyz/anchor';
import { assert } from 'chai';
import {
  fundSol,
  getMintPDA,
  getMetadataPDA,
  getPoolPDA,
  getTickerString,
  getSOLBalance,
  getSPLBalance,
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
      await fundSol(user.publicKey);
    }
  });

  it('Can launch token', async () => {
    const creator = userA;
    const tokenInfo = tokens[0];
    const mint = getMintPDA(tokenInfo.symbol);
    const metadata = getMetadataPDA(mint);
    try {
      await program.methods
        .initializePool(tokenInfo.symbol, tokenInfo.name, tokenInfo.uri)
        .accounts({
          signer: creator.publicKey,
          metadata,
        })
        .signers([creator])
        .rpc();
      const poolPDA = getPoolPDA(tokenInfo.symbol);
      const pool = await program.account.pool.fetch(poolPDA);
      createdPools.push(pool);
    } catch (err) {
      console.log('err', err);
      assert.fail();
    }
  });

  it('Can buy on bonding curve', async () => {
    const buyer = userB;
    const pool = createdPools[0];
    const ticker = getTickerString(pool.ticker);
    const BUY_AMOUNT = 100_000_000_000;
    try {
      const buyerSolBalBefore = await getSOLBalance(buyer.publicKey);
      const buyerTokenAccount = await anchor.utils.token.associatedAddress(
        {
          mint: getMintPDA(ticker),
          owner: buyer.publicKey,
        }
      );
      const buyerTokenBalBefore = await getSPLBalance(buyerTokenAccount);
      await program.methods
        .buy(ticker, new anchor.BN(BUY_AMOUNT))
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
    const ticker = getTickerString(pool.ticker);
    const SELL_AMOUNT = 100_000_000_000;
    try {
      const sellerSolBalBefore = await getSOLBalance(seller.publicKey);
      const sellerTokenAccount =
        await anchor.utils.token.associatedAddress({
          mint: getMintPDA(ticker),
          owner: seller.publicKey,
        });
      const sellerTokenBalBefore = await getSPLBalance(sellerTokenAccount);
      await program.methods
        .sell(ticker, new anchor.BN(SELL_AMOUNT))
        .accounts({
          seller: seller.publicKey,
        })
        .signers([seller])
        .rpc();
      const sellerTokenBalAfter = await getSPLBalance(sellerTokenAccount);
      const sellerSolBalAfter = await getSOLBalance(seller.publicKey);
      assert.ok(sellerSolBalAfter > sellerSolBalBefore);
      assert.ok(sellerTokenBalAfter < sellerTokenBalBefore);
    } catch (err) {
      console.log('err buying', err);
      assert.fail();
    }
  });

  it('Can not close pool early', async () => {
    const creator = userA;
    const pool = createdPools[0];
    const ticker = getTickerString(pool.ticker);
    try {
      await program.methods
        .close(ticker)
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
});

// const buyTokens = async (buyer: any, ticker: string, amount: number) => {
//   const poolPDA = getPoolPDA(ticker);
//   const poolFromProgram = await program.account.pool.fetch(poolPDA);
//   const buyerTokenAccount = await anchor.utils.token.associatedAddress({
//     mint: getMintPDA(ticker),
//     owner: buyer.publicKey,
//   });

//   const priceInSol = getSol(poolFromProgram.tokPrice.toNumber());
//   console.log(
//     `Buyer buying ${amount} tokens for ${priceInSol} SOL per token`
//   );
//   const [ammConfigAddress, _] = await getAmmConfigAddress(
//     0,
//     raydiumSwapProgramId
//   );
//   const txn = await program.methods
//     .buy(ticker, new anchor.BN(amount))
//     .accounts({
//       buyer: buyer.publicKey,
//       cpSwapProgram: raydiumSwapProgramId,
//       ammConfig: ammConfigAddress,
//       token1Mint: SOL_MINT,
//       creatorToken1: buyer.publicKey,
//     })
//     .signers([buyer])
//     .rpc();
//   return txn;
// };

// const sellTokens = async (seller: any, ticker: string, amount: number) => {
//   const poolPDA = getPoolPDA(ticker);
//   const poolFromPogram = await program.account.pool.fetch(poolPDA);
//   const sellerTokenAccount = await anchor.utils.token.associatedAddress({
//     mint: getMintPDA(ticker),
//     owner: seller.publicKey,
//   });
//   const priceInSol = getSol(poolFromPogram.tokPrice.toNumber());
//   console.log(
//     `Seller selling ${amount} tokens at ${priceInSol} SOL per token`
//   );
//   const txn = await program.methods
//     .sell(ticker, new anchor.BN(amount))
//     .accounts({
//       seller: seller.publicKey,
//       sellerTokenAccount,
//     })
//     .signers([seller])
//     .rpc();
//   return txn;
// };

// describe('memetik', () => {
//   const createdPools: PoolFromProgram[] = [];

//   before(async () => {
//     for (const user of users) {
//       await fundSol(user.publicKey);
//     }
//     await mintFake();
//   });

//   it('Can launch token', async () => {
//     const token = tokens[0];
//     try {
//       const creator = userA;
//       const mint = getMintPDA(token.symbol);
//       const metadata = getMetadataPDA(mint);
//       const txn = await program.methods
//         .initialize(token)
//         .accounts({
//           signer: creator.publicKey,
//           metadata,
//         })
//         .signers([creator])
//         .rpc();
//       await waitForTxnConfrimation(txn);
//       const pool = await program.account.pool.fetch(
//         getPoolPDA(token.symbol)
//       );
//       const escrowPDA = getEscrowPDA(token.symbol);
//       const escrowSolBalance = await getSOLBalance(escrowPDA);
//       const escrowAcc = await program.account.poolEscrow.fetch(escrowPDA);
//       const creatorTokenAccount =
//         await anchor.utils.token.associatedAddress({
//           mint: mint,
//           owner: creator.publicKey,
//         });
//       const creatorTokenBalance = await getSPLBalance(creatorTokenAccount);
//       assert.ok(pool);
//       assert.ok(creatorTokenBalance === 0);
//       assert.ok(escrowSolBalance > 0);
//       assert.ok(escrowSolBalance >= escrowAcc.balance.toNumber());
//       createdPools.push(pool);
//       console.log('Token Price at creation:', pool.tokPrice);
//     } catch (err) {
//       console.log('Create token', err);
//       assert.fail('Transaction failed');
//     }
//   });

//   it('Can fetch pool', async () => {
//     const pool = createdPools[0];
//     const poolFromProgram = await program.methods
//       .getPool(pool.ticker)
//       .view();
//     assert.ok(poolFromProgram.ticker === pool.ticker);
//   });

//   it('Can launch another token', async () => {
//     const token = tokens[1];
//     const creator = userA;
//     const mint = getMintPDA(token.symbol);
//     const metadata = getMetadataPDA(mint);
//     try {
//       const txn = await program.methods
//         .initialize(token)
//         .accounts({
//           signer: creator.publicKey,
//           metadata,
//         })
//         .signers([creator])
//         .rpc();
//       await waitForTxnConfrimation(txn);
//       const pool = await program.account.pool.fetch(
//         getPoolPDA(token.symbol)
//       );
//       const creatorTokenAccount =
//         await anchor.utils.token.associatedAddress({
//           mint: mint,
//           owner: creator.publicKey,
//         });
//       const creatorTokenBalance = await getSPLBalance(creatorTokenAccount);
//       assert.ok(pool);
//       assert.ok(creatorTokenBalance === 0);
//       createdPools.push(pool);
//       console.log('Token Price at creation:', pool.tokPrice);
//     } catch (err) {
//       console.log('Create token', err);
//       assert.fail('Transaction failed');
//     }
//   });

//   it('can NOT close pool before maturity', async () => {
//     const creator = userA;
//     const pool = createdPools[0];
//     try {
//       await program.methods
//         .close(pool.ticker)
//         .accounts({
//           creator: creator.publicKey,
//         })
//         .signers([creator])
//         .rpc();
//       assert.fail('Transaction failed');
//     } catch (err) {
//       assert.ok(err?.error?.errorCode?.code === 'PoolNotMatured');
//     }
//   });

//   it('Can buy tokens', async () => {
//     const buyer = userB;
//     const amount = 1;
//     const pool = createdPools[0];
//     try {
//       const buyerTokenAccount = await anchor.utils.token.associatedAddress(
//         {
//           mint: getMintPDA(pool.ticker),
//           owner: buyer.publicKey,
//         }
//       );
//       const solBalBefore = await getSOLBalance(buyer.publicKey);
//       const tokBalBefore = await getSPLBalance(buyerTokenAccount);
//       await buyTokens(buyer, pool.ticker, amount);
//       const solBalAfter = await getSOLBalance(buyer.publicKey);
//       const tokBalAfter = await getSPLBalance(buyerTokenAccount);
//       assert.ok(solBalAfter < solBalBefore);
//       assert.ok(tokBalAfter > tokBalBefore);
//     } catch (err) {
//       console.log('Can buy err', err);
//       assert.fail('Transaction failed');
//     }
//   });
//   it('Token price increases with demand', async () => {
//     const buyer = userC;
//     const pool = createdPools[0];
//     const poolPDA = getPoolPDA(pool.ticker);
//     try {
//       const buyerTokenAccount = await anchor.utils.token.associatedAddress(
//         {
//           mint: getMintPDA(pool.ticker),
//           owner: buyer.publicKey,
//         }
//       );
//       const totalAmountToBy = getLamports(250);
//       const batchAmount = totalAmountToBy / 12;
//       let amountPurchased = 0;
//       while (amountPurchased < totalAmountToBy) {
//         const pool = await program.account.pool.fetch(poolPDA);
//         let tokPriceBefore = pool.tokPrice;
//         let solBalBefore = await getSOLBalance(buyer.publicKey);
//         let tokBalBefore = await getSPLBalance(buyerTokenAccount);
//         await buyTokens(buyer, pool.ticker, batchAmount);
//         const poolAfter = await program.account.pool.fetch(poolPDA);
//         const solBalanceAfter = await getSOLBalance(buyer.publicKey);
//         const tokenBalanceAfter = await getSPLBalance(buyerTokenAccount);
//         console.log('solBalBefore', solBalBefore);
//         console.log('solBalanceAfter', solBalanceAfter);
//         console.log('tokBalBefore', tokBalBefore);
//         console.log('tokenBalanceAfter', tokenBalanceAfter);
//         console.log('New Token price:', poolAfter.tokPrice.toNumber());
//         assert.ok(solBalanceAfter < solBalBefore);
//         assert.ok(tokenBalanceAfter > tokBalBefore);
//         assert.ok(
//           poolAfter.tokPrice.toNumber() >= tokPriceBefore.toNumber()
//         );
//         tokPriceBefore = poolAfter.tokPrice;
//         solBalBefore = solBalanceAfter;
//         amountPurchased += batchAmount;
//       }
//     } catch (err) {
//       console.log('Can buy demand error', err);
//       assert.fail('Transaction failed');
//     }
//   });
//   it('Can sell tokens', async () => {
//     const seller = userB;
//     const pool = createdPools[0];
//     const poolPDA = getPoolPDA(pool.ticker);
//     try {
//       const poolBefore = await program.account.pool.fetch(poolPDA);
//       const sellerTokenAccount =
//         await anchor.utils.token.associatedAddress({
//           mint: getMintPDA(pool.ticker),
//           owner: seller.publicKey,
//         });
//       const sellerSolBalBefore = await getSOLBalance(seller.publicKey);
//       const sellerTokBalBefore = await getSPLBalance(sellerTokenAccount);
//       const priceBefore = poolBefore.tokPrice;
//       await sellTokens(seller, pool.ticker, sellerTokBalBefore);
//       const sellerSolBalAfter = await getSOLBalance(seller.publicKey);
//       const sellerTokBalAfter = await getSPLBalance(sellerTokenAccount);
//       const poolAfter = await program.account.pool.fetch(poolPDA);
//       assert.ok(sellerSolBalAfter >= sellerSolBalBefore);
//       assert.ok(sellerTokBalAfter < sellerTokBalBefore);
//       assert.ok(poolAfter.tokPrice.toNumber() <= priceBefore.toNumber());
//     } catch (err) {
//       console.log('Can sell err', err);
//       assert.fail('Transaction failed');
//     }
//   });
//   it('Token price decresses when supply decreases', async () => {
//     const seller = userC;
//     const pool = createdPools[0];
//     const poolPDA = getPoolPDA(pool.ticker);
//     try {
//       const sellerTokenAccount =
//         await anchor.utils.token.associatedAddress({
//           mint: getMintPDA(pool.ticker),
//           owner: seller.publicKey,
//         });
//       const totalAmountToSell = await getSPLBalance(sellerTokenAccount);
//       const batchAmount = totalAmountToSell / 5;
//       let amountSold = 0;
//       while (amountSold <= totalAmountToSell) {
//         const pool = await program.account.pool.fetch(poolPDA);
//         let tokPriceBefore = pool.tokPrice;
//         let solBalBefore = await getSOLBalance(seller.publicKey);
//         let tokBalBefore = await getSPLBalance(sellerTokenAccount);
//         if (tokBalBefore - 1 < batchAmount) break;
//         await sellTokens(seller, pool.ticker, batchAmount);
//         const poolAfter = await program.account.pool.fetch(poolPDA);
//         const solBalanceAfter = await getSOLBalance(seller.publicKey);
//         const tokenBalanceAfter = await getSPLBalance(sellerTokenAccount);
//         assert.ok(tokenBalanceAfter < tokBalBefore);
//         assert.ok(solBalanceAfter > solBalBefore);
//         assert.ok(
//           poolAfter.tokPrice.toNumber() <= tokPriceBefore.toNumber()
//         );
//         solBalBefore = solBalanceAfter;
//         tokPriceBefore = poolAfter.tokPrice;
//         amountSold += batchAmount;
//         tokBalBefore = tokenBalanceAfter;
//         console.log('tok bal after', tokenBalanceAfter);
//         console.log('sol bal after', solBalanceAfter);
//       }
//     } catch (err) {
//       console.log('Selling demand decreases', err);
//       assert.fail('Transaction failed');
//     }
//   });

//   it('bad actor can NOT close pool', async () => {
//     const badActor = userB;
//     const pool = createdPools[0];
//     const maturityTimeStampMs = pool.maturityTime.toNumber() * 1000;
//     try {
//       await waitUntilTime(maturityTimeStampMs);
//       await program.methods
//         .close(pool.ticker)
//         .accounts({
//           creator: badActor.publicKey,
//         })
//         .signers([badActor])
//         .rpc();
//       assert.fail('Transaction failed');
//     } catch (err) {
//       assert.ok(err?.error?.errorCode?.code === 'NotPoolCreator');
//     }
//   });

//   it('creator can close pool after maturity date', async () => {
//     const creator = userA;
//     const pool = createdPools[0];
//     const maturityTimeStampMs = pool.maturityTime.toNumber() * 1000;
//     const creatorBalBefore = await getSOLBalance(creator.publicKey);
//     try {
//       await waitUntilTime(maturityTimeStampMs);
//       await program.methods
//         .close(pool.ticker)
//         .accounts({
//           creator: creator.publicKey,
//         })
//         .signers([creator])
//         .rpc();
//       const creatorBalAfter = await getSOLBalance(creator.publicKey);
//       assert.ok(creatorBalAfter > creatorBalBefore);
//     } catch (err) {
//       console.log('Close pool error', err);
//       assert.fail('Transaction failed');
//     }
//   });
// });
