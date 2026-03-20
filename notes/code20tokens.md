Code 20 coin proposition:

# Code 20 Economy Proposal

There is a central treasury which issues contracts for each solution width `w`. These contracts will pay out 1 token to the holder each time a discovery is made of witdh `w`. A contract is priced at `p_i * r ^ n` tokens, where `p_i` is the initial price, `r` is the factor of increase, and `n` is the number of contract that have been sold.

When a discovery is made, the discover should buy contracts while the price remains below 1 token. Then, the solution should be submitted so that the holders of the width of that solution are paid out. The discoverer may now try to sell the contracts to the open market, but they will be worth much less (both because he bought them for such a high price (as close as possible to 1 token), and also becasue it decreases the probability of another discovery being found).

If two discoveries are made simultaneously, the discoverer will choose to buy contracts while the price remains below 2 tokens.

The treasury cannot be considered a pool, because when a solution is made, there is more payout than there was investment. It would better be positioned as a bank, which used the tokens from outstanding contracts to pay out new discoveries. However, it is possible that if a single width produced an extremely large number of results (which exceeded the total tokens of the rest of the economy), it would go bankrupt. Therefore, I think the best interpretation is that tokens are destroyed when a contract is bought, and new tokens are minted when it pays out.


# Code 29 Checker Proposal

A brute force checker does not work well, because long-lasting solutions can be engineered. For example, this lasts over 10,000 steps:
```
Join[IntegerDigits[187, 2], ConstantArray[0, 1200], IntegerDigits[189, 2]]
```

Even periodic solutions can be abused, for example if you allow a person to input an arbitrarily large period, then they can use up all of your compute. If you try to manually check for a period, then you end up with the original problem.

Instead, it might be best to suggest a proof of persistence, in the form of a program in some proof checking language. It is fairly easy to prove that a certain periodic solution is persistent, but it is much harder to prove that a non-periodic solution (if such a solution exists) 
