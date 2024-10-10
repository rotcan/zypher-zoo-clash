import { HardhatUserConfig } from "hardhat/config";
import "@nomicfoundation/hardhat-toolbox";
import "@nomicfoundation/hardhat-ethers";
import "@nomicfoundation/hardhat-chai-matchers";

const {secret}=require("./secret.json");
const config: HardhatUserConfig = {
  solidity:{
    version: "0.8.24",
    settings: {
      optimizer: {
        enabled: true,
        runs: 1,
        details: {
          yul: true,
          yulDetails: {
            stackAllocation: true,
            optimizerSteps: "dhfoDgvulfnTUtnIf"
          }
        }
      }
    },
  },
  networks:{
    testnet:{
      url:"https://opbnb-testnet-rpc.bnbchain.org",
      chainId:5611,
      accounts:[secret],
    }
  }
  
};

export default config;
