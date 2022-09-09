// You can discretely check your connected account (not listen to chain changes and such..)

async function getProvider() {
    //console.log(`connecting Metamask from browser`);
    const provider = new ethers.providers.Web3Provider(window.ethereum);
    await provider.send("eth_requestAccounts", []);
    //if(provider) Object.entries(provider).forEach(keyValuePair => {console.log("  ",...keyValuePair)})    
    return provider;
}

async function getSigner() {
  const provider = await getProvider();
  const signer = provider.getSigner();
  //if(signer) Object.entries(signer).forEach(keyValuePair => {console.log("  ",...keyValuePair)})
  return signer;
}

export async function getProviderData() {
  const provider = await getProvider();
  let providerWeb3 = provider.provider;
  // should return an object that i can parse into a ethers(rs)::JsonRpcProvider
  return providerWeb3;
}

export async function signMessage() {
    const signer = await getSigner();
    const flatSignature = await signer.signMessage("Hello World");
    console.log(`signed: ${flatSignature}`);
    return flatSignature;
}

export async function setRinkeby() {
  try {
      await window.ethereum.request({
        method: 'wallet_switchEthereumChain',
        params: [
          {
            chainId: "0x4",
          },
        ],
      });
      return true;
    } catch (switchError) {
      console.log("error during chain change");
  }
}

////////////////////////////////////////////////////////////////////// ERC20
export async function transfer(token_address, recipient, amount, decimals) {
  let abi = [
      'function transfer(address recipient, uint256 amount) external returns(bool)',
  ];
  const signer = await getSigner();
  const contract = new ethers.Contract(token_address, abi, signer);
  const amountParsed = ethers.utils.parseUnits(amount, decimals);
  //console.log(`tx ${tx.hash}`);
  const tx = await contract.transfer(recipient, amountParsed);
  //console.log(`tx ${tx.hash}`);
  return tx.hash;
}

