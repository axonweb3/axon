/**
 * Get the number of most recent block
 *
 * cmd:
 * curl <rpcUrl> -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params": [],"id":2}'
 * 
 * @param {String} rpcUrl default to http://localhost:8000
 * @returns {Number} the current block number the client is on.
 */
async function getLatestBlockNum(rpcUrl) {
  const rawResponse = await fetch(rpcUrl || 'http://localhost:8000', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: '{"jsonrpc":"2.0", "method":"eth_blockNumber", "params": [], "id":42}'
  });
  const content = await rawResponse.json();

  const tipBlockNumber = Number(content.result);
  return tipBlockNumber;
}

const asyncSleep = (ms = 0) => {
  return new Promise((r) => setTimeout(r, ms));
};

/**
 * wait N blocks passed
 *
 * @param {string} [rpcUrl]
 * @param {number} [waitBlocks=1] 
 * @param {undefined} [start=undefined] 
 */
async function waitXBlocksPassed(rpcUrl, waitBlocks = 2, start = undefined) {
  let curBlockNum = await getLatestBlockNum(rpcUrl);
  const startBlockNum = start || curBlockNum;
  const endBlockNum = startBlockNum + waitBlocks;

  while (curBlockNum < endBlockNum) {
    console.log(`The current block number is ${curBlockNum}`)
    console.log(`Wait until Block#${endBlockNum} produced...`);
    await asyncSleep(2000);
    curBlockNum = await getLatestBlockNum(rpcUrl);
  }
}

module.exports = { getLatestBlockNum, waitXBlocksPassed };
