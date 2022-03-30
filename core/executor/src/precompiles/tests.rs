use evm::Context;
use rand::random;
use sha2::Digest;

use protocol::codec::hex_decode;

use crate::precompiles::{
    Blake2F, EcAdd, EcMul, EcPairing, Identity, PrecompileContract, Ripemd160, Sha256,
};

macro_rules! test_precompile {
    ($ty: ident, $input: expr, $output: expr, $expect_gas_cost: expr) => {
        let gas_cost = <$ty as PrecompileContract>::gas_cost($input);
        let resp = <$ty as PrecompileContract>::exec_fn($input, None, &mock_context(), false);
        assert!(resp.is_ok());
        assert_eq!(resp.unwrap().output, $output);
        assert_eq!(gas_cost, $expect_gas_cost);
    };

    ($ty: ident, $input: expr, $gas_limit: expr, $expect_gas_cost: expr, $err: expr) => {
        let gas_cost = <$ty as PrecompileContract>::gas_cost($input);
        let resp =
            <$ty as PrecompileContract>::exec_fn($input, Some($gas_limit), &mock_context(), false);
        assert!(resp.is_err());
        assert_eq!(resp, $err);
        assert_eq!(gas_cost, expect_gas_cost);
    };
}

fn mock_context() -> Context {
    Context {
        address:        Default::default(),
        caller:         Default::default(),
        apparent_value: Default::default(),
    }
}

fn rand_bytes(len: usize) -> Vec<u8> {
    (0..len).map(|_| random::<u8>()).collect()
}

#[test]
fn test_sha256() {
    let input = rand_bytes(100);
    let mut hasher = sha2::Sha256::default();
    hasher.update(&input);
    let output = hasher.finalize().to_vec();

    test_precompile!(Sha256, &input, output, 108);
}

#[test]
fn test_ripemd160() {
    let input = &[0xff];
    let output = hex_decode("2c0c45d3ecab80fe060e5f1d7057cd2f8de5e557").unwrap();
    test_precompile!(Ripemd160, input, output, 720);
}

#[test]
fn test_identity() {
    let data = rand_bytes(16);
    test_precompile!(Identity, &data, data, 18);
}

#[test]
fn test_ec_add() {
    let input = &hex_decode("089142debb13c461f61523586a60732d8b69c5b38a3380a74da7b2961d867dbf2d5fc7bbc013c16d7945f190b232eacc25da675c0eb093fe6b9f1b4b4e107b3625f8c89ea3437f44f8fc8b6bfbb6312074dc6f983809a5e809ff4e1d076dd5850b38c7ced6e4daef9c4347f370d6d8b58f4b1d8dc61a3c59d651a0644a2a27cf").unwrap();
    let output = hex_decode("0a6678fd675aa4d8f0d03a1feb921a27f38ebdcb860cc083653519655acd6d79172fd5b3b2bfdd44e43bcec3eace9347608f9f0a16f1e184cb3f52e6f259cbeb").unwrap();
    test_precompile!(EcAdd, input, output, 150);

    let input = &hex_decode("23f16f1bcc31bd002746da6fa3825209af9a356ccd99cf79604a430dd592bcd90a03caeda9c5aa40cdc9e4166e083492885dad36c72714e3697e34a4bc72ccaa21315394462f1a39f87462dbceb92718b220e4f80af516f727ad85380fadefbc2e4f40ea7bbe2d4d71f13c84fd2ae24a4a24d9638dd78349d0dee8435a67cca6").unwrap();
    let output = hex_decode("013f227997b410cbd96b137a114f5b12d5a3a53d7482797bcd1f116ff30ff1931effebc79dee208d036553beae8ca71afb3b4c00979560db3991c7e67c49103c").unwrap();
    test_precompile!(EcAdd, input, output, 150);

    let input = &hex_decode("0341b65d1b32805aedf29c4704ae125b98bb9b736d6e05bd934320632bf46bb60d22bc985718acbcf51e3740c1565f66ff890dfd2302fc51abc999c83d8774ba08ed1b33fe3cd3b1ac11571999e8f451f5bb28dd4019e58b8d24d91cf73dc38f11be2878bb118612a7627f022aa19a17b6eb599bba4185df357f81d052fff90b").unwrap();
    let output = hex_decode("0e9e24a218333ed19a90051efabe246146a6d5017810140ef7e448030539038a230598b7d4127f5b4fd971820084c632ca940b29fcf30139cd1513bbbbf3a3dc").unwrap();
    test_precompile!(EcAdd, input, output, 150);

    let input = &hex_decode("279e2a1eee50ae1e3fe441dcd58475c40992735644de5c8f6299b6f0c1fe41af21b37bd13a881181d56752e31cf494003a9d396eb908452718469bc5c75aa8071c35e297f7c55363cd2fd00d916c67fad3bdea15487bdc5cc7b720f3a2c8b776106c2a4cf61ab73f91f2258f1846b9be9d28b9a7e83503fa4f4b322bfc07223c").unwrap();
    let output = hex_decode("22f8aa414eb0b9b296bed3fb355804e92ec0af419d9906335f50f032d87a8bf82643f41b228310b816c784c2c54dcfadeaa328b792dbe0d0e04741cd61dac155").unwrap();
    test_precompile!(EcAdd, input, output, 150);

    let input = &hex_decode("0af6f1fd0b29a4f055c91a472f285e919d430a2b73912ae659224e24a458c65e2c1a52f5abf3e86410b9a603159b0bf51abf4d72cbd5e8161a7b5c47d60dfe571f752f85cf5cc01b2dfe279541032da61c2fcc8ae0dfc6d4253ba9b5d3c858231d03a84afe2a9f595ab03007400ccd36a2c0bc31203d881011dfc450c39b5abe").unwrap();
    let output = hex_decode("1e51b9f09d8fc2e4ca11602326c2cfe191c6c6a47874526e80051197e9f6af842282e508ca489bf881e25cf9590151ff5cf94fa523683a0718d87abcc4d4a16f").unwrap();
    test_precompile!(EcAdd, input, output, 150);

    let input = &hex_decode("16a9fe4620e58d70109d6995fe5f9eb8b3d533280cc604a333dcf0fa688b62e20b972bf2daef6c10a41db685c2417b6f4362032421c8466277d3271b6e8706a809ad61a8a83df55f6cd293cd674338c35dbb32722e9db2d1a3371b43496c05fa09c73b138499e36453d67a2c9b543c2188918287c4eef2c3ccc9ebe1d6142d01").unwrap();
    let output = hex_decode("005a68cc13a108287aa3ca0bd8bef95096ef22668e15c87f7cbe0167cd1cdc930359b9b2dd28843838cf74cb4af2cfd656690a7f73de771b891142db22fa61fb").unwrap();
    test_precompile!(EcAdd, input, output, 150);

    let input = &hex_decode("001faaf97b965ffa633612b7c8f9f4be0b286b19662e5cbe6878019d8ba1382b16567ced7a7ee5c272bbc378a95c2436fb0c6133649c77e55a708b28419b5cac0750d51706ced69621c8e4ba1758ba90c39ba8b3b50507bfa545ace1737e360e283d609cd67a291fc3d720c5b1113eececba4ca31d58a1319d6a5a2fa89608f9").unwrap();
    let output = hex_decode("044fe3c480840e5a8f544efd28a8bf3246f0741a8c61c3116e93d84773399c8b26c5b695120cd724aa2a5f4dfd3042c07f752be4c4a8b750398109d80f4772eb").unwrap();
    test_precompile!(EcAdd, input, output, 150);
}

#[test]
fn test_ec_mul() {
    let input = &hex_decode("089142debb13c461f61523586a60732d8b69c5b38a3380a74da7b2961d867dbf2d5fc7bbc013c16d7945f190b232eacc25da675c0eb093fe6b9f1b4b4e107b36ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();
    let output = hex_decode("0bf982b98a2757878c051bfe7eee228b12bc69274b918f08d9fcb21e9184ddc10b17c77cbf3c19d5d27e18cbd4a8c336afb488d0e92c18d56e64dd4ea5c437e6").unwrap();
    test_precompile!(EcMul, input, output, 6000);

    let input = &hex_decode("25f8c89ea3437f44f8fc8b6bfbb6312074dc6f983809a5e809ff4e1d076dd5850b38c7ced6e4daef9c4347f370d6d8b58f4b1d8dc61a3c59d651a0644a2a27cfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();
    let output = hex_decode("18a902ac147b2951531770c7c18a25e3dd87765e23f7e0c4e9d62b624a6e37450288473776e7e99b2aaa27e8f4656ea9ce5e634fd1ca1aab45315199ecaced2e").unwrap();
    test_precompile!(EcMul, input, output, 6000);

    let input = &hex_decode("23f16f1bcc31bd002746da6fa3825209af9a356ccd99cf79604a430dd592bcd90a03caeda9c5aa40cdc9e4166e083492885dad36c72714e3697e34a4bc72ccaaffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();
    let output = hex_decode("0c6a880ffdd0737c53bfec9b65c9098a3298747bd4e5fd07026661b4cb804331116aeec88e11f49753df224c60c4bd8b8bc0a98b8d50f24ce64475268d227f4c").unwrap();
    test_precompile!(EcMul, input, output, 6000);

    let input = &hex_decode("0341b65d1b32805aedf29c4704ae125b98bb9b736d6e05bd934320632bf46bb60d22bc985718acbcf51e3740c1565f66ff890dfd2302fc51abc999c83d8774baffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();
    let output = hex_decode("15bd6ea71fd264e1bfb04eb6d97b4f3686c5bf36f91356fc13ddde3494e172d90b3f8392fd4cdd5d542887ea4ee0274835bf37b58edf927ef242b8704af52e92").unwrap();
    test_precompile!(EcMul, input, output, 6000);

    let input = &hex_decode("1f752f85cf5cc01b2dfe279541032da61c2fcc8ae0dfc6d4253ba9b5d3c858231d03a84afe2a9f595ab03007400ccd36a2c0bc31203d881011dfc450c39b5abeffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();
    let output = hex_decode("0aa7b6fda656f23eab50e36db0519cdf79f4624d417253085907ebfd9aef38a414cdd2edce2b313fc6dd390628ac9fac910841706d55f9af2a064548694dc05c").unwrap();
    test_precompile!(EcMul, input, output, 6000);
}

#[test]
fn test_ec_pair() {
    let input = &hex_decode("089142debb13c461f61523586a60732d8b69c5b38a3380a74da7b2961d867dbf2d5fc7bbc013c16d7945f190b232eacc25da675c0eb093fe6b9f1b4b4e107b3629f2c1dbcc614745f242077001ec9edd475acdab9ab435770d456bd22bbd2abf268683f9b1be0bde4508e2e25e51f6b44da3546e87524337d506fd03c4ff7ce01851abe58ef4e08916bec8034ca62c04cd08340ab6cc525e61706340926221651b71422869c92e49465200ca19033a8aa425f955be3d8329c4475503e45c00e1").unwrap();
    let output =
        hex_decode("0000000000000000000000000000000000000000000000000000000000000000").unwrap();
    test_precompile!(EcPairing, input, output, 79000);

    let input = &hex_decode("23f16f1bcc31bd002746da6fa3825209af9a356ccd99cf79604a430dd592bcd90a03caeda9c5aa40cdc9e4166e083492885dad36c72714e3697e34a4bc72ccaa2e832e2bd419280d247658bb64323d59bbf47df41aa729d2168a272d66e306ff18ab999098bc5b30758183a160fcca776562d9a9370278aee9e6f71053e9358f0edd6252e0584efe53db6b3c40d1976d3849f08db15d39a0d7a6e327fc67f45e24925638e68e59cc22218917c61d2934c4e6353e2f62178a09627aed68c4e57a").unwrap();
    let output =
        hex_decode("0000000000000000000000000000000000000000000000000000000000000000").unwrap();
    test_precompile!(EcPairing, input, output, 79000);

    let input = &hex_decode("0af6f1fd0b29a4f055c91a472f285e919d430a2b73912ae659224e24a458c65e2c1a52f5abf3e86410b9a603159b0bf51abf4d72cbd5e8161a7b5c47d60dfe5712fa10f98bc3c4faa2a9729408e92a8b4e6d304abc539cc396820f696f23ecbb2b30d01fe71a3550fa6780e70305d28ea686b72aff7d23cd550f886fcd60f6b30d1142c81b9b27e329c498e4b241fa2ab0e0c62170cbe2a1e642589e4ff6ca54069ec036f519726e40596ede6cd47577c5cc24746754280add50bf87fe7404c8").unwrap();
    let output =
        hex_decode("0000000000000000000000000000000000000000000000000000000000000000").unwrap();
    test_precompile!(EcPairing, input, output, 79000);
}

#[test]
fn test_blake2f() {
    let input = &hex_decode("0000000048c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000001").unwrap();
    let output = hex_decode("08c9bcf367e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d282e6ad7f520e511f6c3e2b8c68059b9442be0454267ce079217e1319cde05b").unwrap();
    test_precompile!(Blake2F, input, output, 144);

    let input = &hex_decode("0000000c48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000001").unwrap();
    let output = hex_decode("ba80a53f981c4d0d6a2797b69f12f6e94c212f14685ac4b74b12bb6fdbffa2d17d87c5392aab792dc252d5de4533cc9518d38aa8dbf1925ab92386edd4009923").unwrap();
    test_precompile!(Blake2F, input, output, 144);
}
