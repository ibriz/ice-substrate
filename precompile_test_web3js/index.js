const Web3 = require('web3')
const chain = 'ws://127.0.0.1:9944'
const web3 = new Web3(chain) 

const PRECOMPILE_TEST_ADDR = '0x0000000000000000000000000000000000000419'

// web3.eth.accounts.wallet.add(pkey);

const data = '0x1234567890123456';
const hex_encoding = Buffer.from(data).toString('hex');

const alice = '0xd43593c715fdd31c61141abd04a99fd6822c8558'; 
const alice_pkey = '0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a';

web3.eth.getBalance(alice).then(function(value) {
	console.log('Alice Balance : ', value); 
	}); 

const npkey = web3.eth.accounts.privateKeyToAccount(alice_pkey); 
web3.eth.accounts.wallet.add(npkey); 

const trans = {
    from : alice,
    to : PRECOMPILE_TEST_ADDR,
    gas : '0x10000',
    data : '0x00000000000000000000000000000013', // represents 19 in plain english 
    };



web3.eth.call(trans).then(function(str) {
	if (str.startsWith('0x')) 
	{
	  let z = ""; 
	  for (var i = 2; i <= str.length - 2; i = i + 2) 
	  {
	    let y = str.slice(i,i+2); 
	    z = z + String.fromCharCode(parseInt(y,16)); 
	  }
	  console.log(z)
	}	
	else 
	  console.log('Invalid response forwarded'); 
	}); 
