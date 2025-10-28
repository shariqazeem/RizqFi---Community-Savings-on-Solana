# Testing RizqFi with Circle USDC

This guide helps you test your RizqFi app with Circle's official USDC token.

## Prerequisites Checklist

Before you start, make sure you have:

- [ ] Phantom or Solflare wallet installed
- [ ] Wallet connected to Solana Devnet
- [ ] Some SOL for transaction fees (get from https://faucet.solana.com/)
- [ ] Testnet USDC from Circle (see below)

## Step 1: Get Testnet USDC

### Method A: Circle Faucet (Quick & Easy)

1. **Visit the faucet**
   ```
   https://faucet.circle.com/
   ```

2. **Select Network**
   - Choose "Solana Devnet"

3. **Enter Your Wallet Address**
   - Copy your Phantom/Solflare wallet address
   - Paste it in the form

4. **Get USDC**
   - Click "Get USDC" button
   - Wait 5-10 seconds
   - You should receive 10 testnet USDC

### Method B: Circle Sandbox (For More USDC)

If you need more testnet USDC:

1. **Sign up for Circle Sandbox**
   ```
   https://app-sandbox.circle.com/
   ```

2. **Create an API Key**
   - Go to "APIs" tab
   - Click "CREATE A KEY"
   - Save the full key (including "SAND_API_KEY:")

3. **Use the Sample App**
   ```
   https://sample-sandbox.circle.com/
   ```

4. **Configure the app**
   - Click settings (gear icon)
   - Paste your API key
   - Save

5. **Add a Bank Account**
   - Go to "POST /businessAccount/banks/wires"
   - Click "PREFILL FORM"
   - Choose "US BANK ACCOUNT"
   - Click "MAKE API CALL"
   - Save the `id` from the response

6. **Get Bank Instructions**
   - Go to "GET /businessAccount/banks/wires/{id}/instructions"
   - Paste the `id` from previous step
   - Click "MAKE API CALL"
   - Save the `trackingRef` and `accountNumber`

7. **Mock a Wire Transfer**
   - Go to "POST /mocks/payments/wire"
   - Fill in:
     - Tracking Ref (from previous step)
     - Account Number (from previous step)
     - Amount (e.g., 1000)
   - Click "MAKE API CALL"

8. **Add SOL Address to Address Book**

   Use this curl command (replace placeholders):

   ```bash
   curl -X POST \
     --url https://api-sandbox.circle.com/v1/addressBook/recipients \
     --header 'accept: application/json' \
     --header "Authorization: Bearer YOUR_SANDBOX_API_KEY" \
     --header 'content-type: application/json' \
     --data '{
       "idempotencyKey": "UNIQUE_KEY_HERE",
       "address": "YOUR_SOLANA_WALLET_ADDRESS",
       "chain": "SOL",
       "metadata": {
         "email": "your@email.com",
         "bns": "",
         "nickname": "My Test Wallet"
       }
     }'
   ```

   Save the `id` from the response.

9. **Create a Payout**
   - Go to "POST /payouts"
   - Fill in:
     - Amount (e.g., 100)
     - Currency: USD
     - Destination: The recipient `id` from previous step
     - Destination Type: address_book
     - Beneficiary Email: your email
   - Click "MAKE API CALL"
   - Save the payout `id`

10. **Monitor the Payout**
    - Go to "GET /payouts/{id}"
    - Paste your payout `id`
    - Click "MAKE API CALL"
    - Wait until status is "complete"
    - Check your wallet - USDC should appear!

## Step 2: Add USDC Token to Your Wallet

### For Phantom Wallet

1. Open Phantom wallet
2. Click the hamburger menu (☰)
3. Go to "Manage Token List"
4. Click "Add Custom Token"
5. Enter token address:
   ```
   4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU
   ```
6. Click "Add Token"
7. You should now see your USDC balance!

### For Solflare Wallet

1. Open Solflare wallet
2. Go to the tokens list
3. Click "Add Token"
4. Enter token address:
   ```
   4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU
   ```
5. Click "Add"
6. You should now see your USDC balance!

## Step 3: Test RizqFi Features

### Test 1: Create a Committee

1. **Launch the app**
   ```bash
   cd rizqfi-frontend
   npm run dev
   ```

2. **Connect your wallet**
   - Click "Connect Wallet"
   - Approve the connection

3. **Create a committee**
   - Click "Create Committee"
   - Fill in:
     - Name: "Test Committee"
     - Monthly Contribution: 10 (USDC)
     - Number of Members: 3
     - Frequency: Monthly
     - Payout Mode: Sequential or Random
   - Click "Create Committee"
   - Approve the transaction in your wallet

4. **Verify success**
   - Check the transaction on Solana Explorer
   - Committee should appear in "My Committees"

### Test 2: Join a Committee

1. **Get the invite code**
   - Copy the invite code from your committee

2. **Open in another wallet** (or use the same for testing)
   - Click "Join Committee"
   - Paste the invite code
   - Click "Join"
   - Approve the transaction

3. **Verify success**
   - Check member count increased
   - New member should appear in committee details

### Test 3: Make a Contribution

1. **Make sure you have USDC**
   - Check your USDC balance
   - Must be >= monthly contribution amount

2. **Contribute to committee**
   - Click on your committee
   - Click "Make Contribution"
   - Approve the transaction

3. **Verify success**
   - Check transaction on Solana Explorer
   - Committee balance should increase
   - Your "Total Contributed" should update

### Test 4: Distribute Payout

1. **Ensure all members have contributed**
   - All members must deposit for current round
   - Committee phase should be "Payout Ready"

2. **Distribute payout**
   - Click "Distribute Payout"
   - Approve the transaction

3. **Verify success**
   - Check recipient's wallet - should receive USDC
   - Committee should move to next round
   - Transaction visible on Solana Explorer

## Troubleshooting

### Issue: "Insufficient USDC balance"

**Solution:**
- Get more USDC from Circle faucet
- Make sure you added the correct USDC token to your wallet
- Check that you're on Solana Devnet, not mainnet

### Issue: "Token account doesn't exist"

**Solution:**
- The app should create it automatically
- Make sure you have enough SOL for rent (~0.002 SOL)
- Try refreshing the page and reconnecting wallet

### Issue: "Transaction failed"

**Solution:**
- Check you have enough SOL for gas fees
- Make sure you're on Solana Devnet
- Try again (sometimes RPC nodes are slow)
- Check browser console for error messages

### Issue: "Can't see USDC in wallet"

**Solution:**
- Add the token manually using the address above
- Switch to Solana Devnet in your wallet
- Refresh your wallet
- Check transaction on Solana Explorer to confirm transfer

### Issue: "Circle faucet not working"

**Solution:**
- Wait a few minutes and try again (rate limits)
- Try using a different wallet address
- Use Circle Sandbox method instead (Method B above)
- Clear browser cache and try again

## Useful Links

### Faucets
- **USDC Faucet**: https://faucet.circle.com/
- **SOL Faucet**: https://faucet.solana.com/

### Explorers
- **USDC Token**: https://explorer.solana.com/address/4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU?cluster=devnet
- **Your Program**: https://explorer.solana.com/address/ABKnVQCt2ATkMivkFux7X3zKnozHzXELc2LiUdZM8vCN?cluster=devnet

### Documentation
- **Circle USDC Docs**: https://developers.circle.com/stablecoins/docs/transfer-usdc-on-solana-devnet
- **Solana Docs**: https://docs.solana.com/

## Expected Results

After successful testing, you should be able to:

- ✅ Get testnet USDC from Circle faucet
- ✅ Create committees using Circle USDC
- ✅ Join committees and create token accounts automatically
- ✅ Make contributions (transfer USDC to vault)
- ✅ Distribute payouts (transfer USDC from vault to recipient)
- ✅ View all transactions on Solana Explorer
- ✅ Verify complete transparency and trustlessness

## For Demo Video

When recording your demo, show:

1. **Getting USDC**
   - Screen record getting USDC from Circle faucet
   - Show it appearing in your wallet

2. **Using the App**
   - Create a committee
   - Join with another wallet
   - Make contributions
   - Distribute payout

3. **Blockchain Verification**
   - Open Solana Explorer
   - Show the transactions
   - Prove it's on-chain

4. **Mention Benefits**
   - "Anyone can get testnet USDC from Circle"
   - "No custom tokens - using real USDC infrastructure"
   - "Same token as mainnet, just for testing"

## Notes for Judges

If judges want to test your app:

1. **Share these instructions** with them
2. **Point them to Circle faucet** - easiest way to get USDC
3. **Provide your app URL** - https://rizqfi.vercel.app
4. **Give them a test committee code** if you have one running

Judges can verify everything on-chain:
- **Smart Contract**: `ABKnVQCt2ATkMivkFux7X3zKnozHzXELc2LiUdZM8vCN`
- **USDC Token**: `4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU`
- **All transactions public** on Solana Explorer

---

**Good luck with testing! 🚀**

If you encounter any issues not covered here, check:
- Browser console for errors
- Solana Explorer for transaction details
- Circle docs for USDC-specific issues
