# Create Mint PDA

Using `anchor-lang` and `anchor-spl` versus emulating `anchor-lang` and `anchor-spl` with
`sealevel-tools`.

## Comparison

| Package                        | Size (B)
| :----------------------------- | -------:
| `create-mint-pda-like-anchor`  | 113,776
| `create-mint-pda-using-anchor` | 225,808

| Package                        | Action             | Compute Units
| :----------------------------- | :----------------- | ------------:
| `create-mint-pda-like-anchor`  | Token Program Mint | 10,904
| `create-mint-pda-using-anchor` | Token Program Mint | 14,824

| Package                        | Action                  | Compute Units
| :----------------------------- | :---------------------- | ------------:
| `create-mint-pda-like-anchor`  | Token 2022 Program Mint | 11,261
| `create-mint-pda-using-anchor` | Token 2022 Program Mint | 15,170

| Package                        | Action                                 | Compute Units
| :----------------------------- | :------------------------------------- | ------------:
| `create-mint-pda-like-anchor`  | Token Program Mint w/ Freeze Authority | 11,210
| `create-mint-pda-using-anchor` | Token Program Mint w/ Freeze Authority | 15,439

| Package                        | Action                                      | Compute Units
| :----------------------------- | :------------------------------------------ | ------------:
| `create-mint-pda-like-anchor`  | Token 2022 Program Mint w/ Freeze Authority | 11,553
| `create-mint-pda-using-anchor` | Token 2022 Program Mint w/ Freeze Authority | 15,785
