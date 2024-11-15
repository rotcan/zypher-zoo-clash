# Game rules: 

Game is played with 20 cards each for the player. In addition there is a public game deck of 20 cards as well. Player gets to mint 20 cards the first time.
The cards represents animals with each animal having points as in how many steps they can run in each round. Cards have rarity with S being the rarest and D being the most common. Along with steps, there are environment slots for each animal which tells if animals takes more/less steps in a round. The environment card which is placed on the table (only one is visible) is used to calculate the effect of it on the animal

In each round player who starts the round has either option to reveal a new card for their deck or reveal a new environment card. Other players can only reveal a new card in their deck and replace the card they played on the table. In next round, the order reverses to give each player a fair opporunity. 

First player to reach the finish line wins the game. By default player requires 12 steps to win the game but it can be changed to any value till 20. 

The winner gets a new random card minted and added to its collection. 

For rust zypher shuffle sdk was missing some functions. So had to add some functions in zypher's uzkge sdk to make it wasm compatible so that game can run in browser
First time using bevy, so took some time to understand how to make the game
Initially the proofs in rust were not working but thanks to zypher it got sorted out which saved a lot of time
The reveal_proof_with_snark function is failing in bevy (maybe issue with bevy runtime), so had to deploy it as a separate service.

# Future additions:

* Have option to allow players to bet on outcome of the game with winner getting the spoils
* Boundary cases like time wasting, tie etc to be handled
* Make the interface more intuitive and easy for players to play the game.
* Have time limit between moves

# Run Application

Build as wasm
```
cargo build --target wasm32-unknown-unknown
```
Run as wasm
```
cargo run --target wasm32-unknown-unknown
```

# * * Note: 

There is an issue with reveal_snark_with_proof function in bevy runtime. As a work around I am running a separate service in backgroundm, which this game calls to reveal proofs. To run that service, please clone and run the application in the repo [https://github.com/rotcan/zypher-test]
