# Exceed program library

Source code of the Exceed protocol's Solana program, which issues and redeems the exUSDC, exSOL and exBTC notes.

## The deployed program

- **Program ID:** `par1tyqusak2f2DXg9RHv78SVHNWXkJLSbtJZQSuWjV` (Solana mainnet)
- **Deployed source:** tag [`mainnet-2026-10-20`](https://github.com/Exceed-Finance/exceed-program-library/tree/mainnet-2026-10-20) (commit `5e66e8b`). `main` holds an earlier version.
- **Verified build:** OtterSec confirms that the program on chain was built from that commit: [verify.osec.io/status/par1tyqusak2f2DXg9RHv78SVHNWXkJLSbtJZQSuWjV](https://verify.osec.io/status/par1tyqusak2f2DXg9RHv78SVHNWXkJLSbtJZQSuWjV)
- **Audit:** [Quantstamp report](https://certificate.quantstamp.com/full/exceed-finance-liquid-staking-early-purchase/cde4c9ed-dfc2-460f-bc2c-780ce622fff7/index.html)

## Notes

- `programs/early-purchase` is not deployed on mainnet.
- `test-keypairs/` holds keys used only by the local test suite. They have no authority over the deployed program.
- The notes are issued under the Exceed Conditions: [exceed.finance/terms-and-conditions.pdf](https://www.exceed.finance/terms-and-conditions.pdf). On-chain addresses for checking the holdings: [exceed.finance/verify](https://www.exceed.finance/verify).

## Contact

Security reports and questions: info@exceed.finance
