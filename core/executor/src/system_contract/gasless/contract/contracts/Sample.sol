// SPDX-License-Identifier: GPL-3.0

// Specify the version of Solidity, and tell the compiler that the compatible version of this code is 0.5.0 to 0.7.0 through Pragmas (https://solidity.readthedocs.io/en/latest/layout-of-source-files.html#pragmas)
pragma solidity >=0.8.0;

// import the SponsorWhitelistControl contract
import "./SponsorWhitelistControl.sol";

// import the SponsorWhitelistControl contract
contract Sample {
    // two State Variables are defined (https://solidity.readthedocs.io/en/latest/structure-of-a-contract.html#state-variables)
    address public minter;
    mapping (address => uint) private balances;

    // Use the SponsorWhitelistControl contract to connect to the system contract.
    // The address must inconsistent with system_contract_address(0x1);
    SponsorWhitelistControl constant private SPONSOR = SponsorWhitelistControl(address(0xFFfffFFfFFfffFfFffFFfFfFfFffFfffFFFFFf01));

    // define the event of `Sent` and the from / to / amount column
    event Sent(address from, address to, uint amount);

    // the constructor of the Coin contract, specify the address of the minter in the constructor
    constructor() {
        // msg.sender is the address of the account signed when deploying the contract, assign this address to minter
        minter = msg.sender;
    }

    // define the mint method, through which tokens can be issued
    function mint(address receiver, uint amount) public {
        require(msg.sender == minter);
        require(amount < 1e60);
        balances[receiver] += amount;
    }

    // define the send method, through which tokens can be transferred to other accounts
    function send(address receiver, uint amount) public {
        require(amount <= balances[msg.sender], "Insufficient balance.");
        balances[msg.sender] -= amount;
        balances[receiver] += amount;
        // trigger the Sent event by the emit method to record the transfer information
        emit Sent(msg.sender, receiver, amount);
    }

    // define the balanceOf method, which is a view type method for querying the account balance
    function balanceOf(address tokenOwner) public view returns(uint balance){
      return balances[tokenOwner];
    }

    // define the addPrivilege method, call the system contract method addPrivilege to add the address to the contract sponsor whitelist
    function addPrivilege(address account) public payable {
        // only the contract owner can add accounts to the whitelist
        require(msg.sender == minter);

        address[] memory a = new address[](1);
        a[0] = account;
        SPONSOR.addPrivilege(a);
    }

    // define the removePrivilege method, call the system contract method removePrivilege to remove the address from the contract sponsor whitelist
    function removePrivilege(address account) public payable {
        // only the contract owner can remove accounts from the whitelist
        require(msg.sender == minter);

        address[] memory a = new address[](1);
        a[0] = account;
        SPONSOR.removePrivilege(a);
    }
}