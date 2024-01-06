# Saga Drop
*Preliminary proof of concept for feedback*

A simple tool to allow token dropers to distribute SPL tokens to holders of a Soul-bound NFT collection (e.g., Saga Genesis tokens). This allows for savings on rent fees for 1,000s of token accounts and allows holders to opt-in to receive tokens.

## How it works
- Dropper creates a token drop by putting the tokens into an escrow ATA.
- Drop has a claim period. During this period, holders of the NFT can claim their tokens. 
- When they claim, a PDA receipt (seeded on the NFT mint) is created to prevent double claiming.
- After the claim period, the dropper can withdraw any unclaimed tokens and reclaim their rent.

## Local Deployment

- Install [Anchor](https://www.anchor-lang.com/docs/installation)
- Clone Repo
- Install Deps `yarn`
- Build prorgram `anchor build`
- Run Tests Locally `anchor test`
