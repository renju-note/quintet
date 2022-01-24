use super::player::*;
use super::point::*;

pub fn from_points(blacks: &Points, whites: &Points) -> u64 {
    let mut result = new();
    for p in blacks.0.iter() {
        result = apply(result, Black, *p)
    }
    for p in whites.0.iter() {
        result = apply(result, White, *p)
    }
    result
}

pub fn new() -> u64 {
    EMPTY_CODE
}

pub fn apply(current: u64, player: Player, p: Point) -> u64 {
    let code = get_code(player, p);
    current ^ code
}

fn get_code(player: Player, p: Point) -> u64 {
    let idx = 2 * (u8::from(p) as usize) + if player.is_black() { 0 } else { 1 };
    CODE_TABLE[idx as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply() {
        let mut code1 = new();
        code1 = apply(code1, Black, Point(7, 7));
        code1 = apply(code1, White, Point(8, 8));
        code1 = apply(code1, Black, Point(9, 8));
        // different order
        let mut code2 = new();
        code2 = apply(code2, Black, Point(7, 7));
        code2 = apply(code2, Black, Point(9, 8));
        code2 = apply(code2, White, Point(8, 8));
        assert_eq!(code1, code2)
    }
}

const TABLE_SIZE: usize = 2 * (RANGE as usize) * (RANGE as usize); // 450

const EMPTY_CODE: u64 = 0x3453e3078a713e56;

const CODE_TABLE: [u64; TABLE_SIZE] = [
    0xf442daf859a9ece6,
    0x35fb310ce93606d2,
    0xfd0bcf8141b17bb5,
    0x3328c121f6b2a8b5,
    0x8ca5c66d431c1a86,
    0x18100c0e3dabc40f,
    0x8c8014baad304ded,
    0xec6ef9ecf8280862,
    0xcb1faff8eb50e4a9,
    0x0d8b34e809f058fe,
    0x119af9d52f02e414,
    0xa65f055abe5655dd,
    0x8c710513ce28706e,
    0x08318c9556917ef9,
    0xb0e6b8108c2f2378,
    0x5e2dc23f47dd2484,
    0x30cf07d207f2205d,
    0x4693659470aef77a,
    0x757cadbf6852e01c,
    0x36bbb42355e4cd44,
    0x53d6e5711b233a44,
    0x03214763dcd2ca94,
    0x50760b4fc2f8c31a,
    0xd1a8228fa4f977fc,
    0x2d75b0128df679fd,
    0x6e920cba28341e16,
    0xf24ce129f0ef755c,
    0x7bad60a632368c0e,
    0xe123efa37458583a,
    0x40801718d7a52b4a,
    0xac9de0406406c7cd,
    0x1d7cc4cac2fb7336,
    0x0dce88d323269848,
    0x1f317960d152e821,
    0xd722590d4ea9acda,
    0xa14c449329402d80,
    0x1107eebd32b55f27,
    0xf445c2bff9d0449f,
    0x7b99934934358334,
    0x2c8aad3d832bd55c,
    0x93bd499e58cfaaf1,
    0x7cb480eeea46f1e4,
    0xf9397c83cad0edb6,
    0xcb7ac0d55d09f7c1,
    0x2f257ed203f22275,
    0x2f6fab66df0243ab,
    0xc72c262523f81f72,
    0x3d8732cfb0452230,
    0x6977db7fae5ddf47,
    0x04493abbc62838fb,
    0x5b4f12b377e467d7,
    0x8dad442965c5e2f0,
    0xc273ebd2912309ce,
    0x05ee0f1752cf8273,
    0x5b6ac48f94fa40b6,
    0xb4b9d93400f61f6c,
    0x77fdd6cd9dbfd137,
    0xe032e472d6ed4a5e,
    0xbe1430d7487b2e71,
    0xb206182c251ce06f,
    0x7ec5cabd2edb7c58,
    0x4b087bf96820aa58,
    0xc31a03ad5134f902,
    0x970892dcf938d985,
    0x4c49d0a9882af00d,
    0x2ae13ed98d4ae5da,
    0x997960eb5aa5e42e,
    0xef6f220452711dd2,
    0x9443976faa3d35ae,
    0xd505a1106bf07cf8,
    0xcecc91a34bbf1f1c,
    0xc4489a521aa26185,
    0xb7ca43c7ffeebf4a,
    0xd3d43b9026243418,
    0xdf2859893311da6a,
    0x2132ca65177ba3d4,
    0x7831771c4e87ac6f,
    0x37cd78d01c63fb8b,
    0xbcea53da1714d8ea,
    0x4e79bd0634001049,
    0xa2ff42af35516b1b,
    0xb1cbab0f2d38b888,
    0xc60b6d6edba59f73,
    0x600325ac374a3568,
    0x1edd96e52c89fd21,
    0x355907e63c1e2cde,
    0x61efba1710a485d4,
    0x9e2469020c50f352,
    0xaab0d784f0c97567,
    0x55d04378e98a275a,
    0xcdec5bbfc1ef4eb4,
    0x34a59aef21b89c84,
    0x280e4693441411cd,
    0x158293fcaab54662,
    0x749f11eb0da1a461,
    0x29fc7368484eaef6,
    0xbb67e7b769dafb12,
    0x006192f4f02c8fd4,
    0x7c354d2a0f29a0ec,
    0xf70cd44840939da4,
    0x884347a7445e8dae,
    0xa6278591a9d63b79,
    0x31bfd9006b9ffc0c,
    0x9bf0f69131ce6fb4,
    0x3d2888c142a97265,
    0x815ff50e882193da,
    0x6e8387209fbb2e34,
    0xd2f156d483100803,
    0xe3a948db598281a3,
    0x64ee96592fbc8854,
    0x295c8c78381abe2b,
    0x1ca6707e96581102,
    0x6bc4aaf05df0d708,
    0x218385121c83689b,
    0x89c2507142c0e1e0,
    0x02701eb06c4e9276,
    0xe6cf4e5611ddae57,
    0xf6fb7142d3eb742f,
    0x6f9232fbdd4cce65,
    0x1b6992ae001adf5f,
    0xbd4f089f4db3c700,
    0x2d4dc6f530fa077a,
    0xc948da7fb3ac346b,
    0x119a4bb4a7388c44,
    0x2ef10e46ba293ac2,
    0x7613f2f679a5fdec,
    0x429631927800454c,
    0xa65f6f5e099cf3c5,
    0x81b0836bcef26765,
    0x22f791efa4e39ec1,
    0xe7193684fed35fbd,
    0xe6800904880c3ac5,
    0x7be1153265f863bf,
    0xa08e598b9c391dd2,
    0x1e29f13fe08ae44d,
    0xbedf871af1393edf,
    0xacfd539d21ef7b73,
    0x896f60d59c359cf3,
    0xfe877d17a22f1215,
    0x62874987d0adefda,
    0x3b3c797f437cd31e,
    0x0d53f5624d20fe70,
    0x8adc6b3d588b7c57,
    0x7eddd30a719d8b00,
    0xc70a764fbd024730,
    0x970fb4f52f529f7f,
    0x52b95486b24fa394,
    0xfee20f2e0270473e,
    0xcb65d5afe9a7dc4f,
    0x7e911f0754fe7413,
    0x9b449755b3be2b85,
    0xe887c135fcdfe82d,
    0x531fa0a599cf1f32,
    0xd439e248f4c2bb46,
    0xce8e0f2130d5048b,
    0x3c23e8e61fe86bb2,
    0x8bbb4318ae43a54d,
    0x84cf0a8d32bde583,
    0xc3b8dbdd1a85a012,
    0x152e9f1744e10603,
    0xdf1b4e4d3cd67a92,
    0x5798f8c0323fc72b,
    0x92fe9c7c2fd63cfb,
    0xed6f598c84aeb2b1,
    0x190597c04f77e343,
    0x5874b07a2d95f18b,
    0x34b5e77a1864db29,
    0xb99ee494f98bb906,
    0xe024cd13b8aed139,
    0x9121892daed38449,
    0x7bf043272d5df49f,
    0xc6dcf0e36268fd84,
    0x341160f6753cf7f6,
    0x3afad93a6847f735,
    0xa38f0c6b4ff2fae1,
    0x9a1803b952a271ac,
    0x276735bf5b0d0e2a,
    0x6841530fc3dff73b,
    0xb52e6bcb6d4496be,
    0x5873d19dcffc17c4,
    0x18f37789b9b55dfe,
    0xbdcf8ba0412fcaf0,
    0xd1c5cf58b6c356db,
    0x609620dd7b638c33,
    0x97406a773a391e17,
    0x5dc696e8c2aaa2cd,
    0x7a39ad08231d6ef7,
    0x788b6b6777617460,
    0x3099b9eefbdd4047,
    0x30d68898e720923c,
    0x2f829316f4912d1c,
    0x532ea164d3565993,
    0x5018f7f16be491be,
    0x87be3805059046e8,
    0x7280c1c860505d05,
    0x1a1ec852d9e4c184,
    0x6d5f07cef93173c1,
    0x8496f243985557e1,
    0xe03d1f410929bef0,
    0xa08485dc43f0dc6b,
    0x528c90951768db6b,
    0xa61efa391c005a13,
    0x5f7a84f07d9a9770,
    0x966893bf16336f3b,
    0xce8070cac72ec717,
    0x02a544f4ae8f1227,
    0x810723347978944c,
    0xd92c03ee7702097e,
    0x6cd6e4763ee0fb59,
    0x2b53638a58459252,
    0x0cde49bf11bf67ef,
    0x059ab50a4c7f9ec5,
    0xe101b0ab9d159c3b,
    0xd1a85ac62544c43f,
    0x50d5af413510581f,
    0x1f53e774c29301fb,
    0xd1744ad9d61aeb47,
    0x8e274032b097982c,
    0x44a261eedb9758b5,
    0x2451e85bacbf5730,
    0x8aefb75a60d9cdaa,
    0xd6f8ad48acc525e4,
    0x9bc11921ecb255b5,
    0xf083fd955d32e44a,
    0x48fc7f9574f673d1,
    0xfe9e65d6256be747,
    0x5e994bb40a8adff5,
    0xb3584c4a4399c343,
    0xaae8d2c1ef9e4f97,
    0x709b6c92cc1bdc6f,
    0x1b92e0d8fe6c4376,
    0x68892816ae422c36,
    0xf4db028809d8f018,
    0xede19735a3aee67b,
    0x2963ec2e3f61e9b0,
    0x0d993408d02777bc,
    0x67399c1924d6549a,
    0x8b957f155e2f80cc,
    0x3804d988e9cdc7f9,
    0x22e915185c4a4edd,
    0x8f2bec8b42d204ab,
    0x64d3c780b1eb7077,
    0xf0c54835244451e1,
    0x4e4524e18eb40241,
    0xf6afe2f2a27966c9,
    0xb0117b983d4ba433,
    0x5bd9352056a7e749,
    0xc2a4e8a498487d4d,
    0x850a5ff8dcdf63a2,
    0xde1b73d23fcac0ec,
    0x69899c8da8d3597e,
    0xa0c7514e67cb1a9d,
    0x85c7b372ac3937fc,
    0x7bf3d50f2bc3600d,
    0x39162ca091051eca,
    0x75081c52fb1f70c0,
    0x6ac22e893a0751c9,
    0xa2e2052bdb3c377a,
    0x631a6c8f954eb1cc,
    0xe64d0aab0cc611f6,
    0x533c054ad753c030,
    0x0b4d617aaea046db,
    0xf120eb2b04faa08b,
    0x9bd629ce0f244f1b,
    0x8f0b2e25ac25764b,
    0x60989a17811cf2b6,
    0x467a7156881b2869,
    0x496af10331282396,
    0xbe9941ffe0001dd8,
    0x534a660c81ee2210,
    0x7ce9bd1a2ee3bc86,
    0x88ae5a587d787521,
    0x27cb1053d20d188c,
    0x11cbba3e82c78088,
    0x8b492f5c5519d6fe,
    0xd313855b9d3ce570,
    0x0960914bf8a4503c,
    0x2fde8aa187a972d3,
    0x8d65b62ba3f6ad86,
    0x2ef2eb531533bcf5,
    0xdfaa92c2484625be,
    0x49907de366b53adb,
    0x5caf70984e243037,
    0xdcca61657f83b896,
    0x00b3a191f9fc21dc,
    0x16810a9787576150,
    0x8904e2867de549e7,
    0x2041ce5aeb1ba022,
    0x91f17069eb0fb62f,
    0xaceb09780f453f9f,
    0x7a352b03250f240d,
    0x7b65691968daed86,
    0x3609d092ca170bf7,
    0xfb77e4f67b78887e,
    0x1b38c2bdd2460123,
    0xa8bd120865c07a01,
    0xf2413b0f0ec97e38,
    0x6df0a14688803e0c,
    0x23bd3fb245523088,
    0xe8b568ec72dd4ceb,
    0xb4dc7bba9f9f02b5,
    0x5e16afb7f689c27f,
    0xcc710bc3650d533e,
    0x2ffdeee4bb5bf634,
    0x54cfdadd8d8694a0,
    0xabfb2aad049205ae,
    0x0457756eb6ce4640,
    0xbcb0169df7d179ef,
    0xfb66d76c02e01e0e,
    0x60c3953080acc226,
    0xf65aed7e0c67d52d,
    0x83ce268a0ecaa7db,
    0x8e584f11b8dd67f5,
    0x75bded3aa74f1ce5,
    0xaa7f80cc2f6d0897,
    0x18154499c430aa7c,
    0x43a25ce5f5b53e6e,
    0xedfca723a12abd47,
    0x96f7bc411cb9b0f9,
    0x74bb817e123206c6,
    0x8a02c867fa783c16,
    0x110f82f67348867c,
    0xb5cc1fdace3d980a,
    0x6ddd9ca34212d25d,
    0x949c0b6b5e65638d,
    0x927c9e5ae27090c0,
    0x8f7d9ecebe548201,
    0xdc83ba89dcd0018b,
    0xc097d2d012df6b6a,
    0x1f7450db2a6e5719,
    0xdc88985645fa56b5,
    0xe1cd6b50725a4eac,
    0xfead047ff51a0a95,
    0x5ab88fcfddbaf5de,
    0x1a23cc874837a6cc,
    0x89441cefdc25916c,
    0x142c551f179c574b,
    0x5f554813487d6a12,
    0xdbdece7912a13c06,
    0xc02baf5c2764cc33,
    0x4b2499b5a004bbb7,
    0x087277942c8df675,
    0xa9677d5dfac97257,
    0xf2e50f0ad1d811e9,
    0x0d36a7e7b3bc7b6b,
    0x2ae9184178b519e9,
    0xd5189e1da6c26873,
    0xa78c086ac943bb9e,
    0x14d6389a5a345050,
    0xcdfbce28887c3dcf,
    0xd63bf3cd8f7ab8ff,
    0x310c988b98f5e5ad,
    0xd2f1cd857295185c,
    0xe43a9974cf52c624,
    0x8080441a459675f5,
    0x523a52dde7dd8fc7,
    0x4718c585a78dea3b,
    0x93c0ea34381d8e2e,
    0x84a361e04ff549f8,
    0x96712cbfb30af042,
    0x191cb994709c394e,
    0x032df5ce8120c29d,
    0xf60927889eae37b1,
    0x05adbdd62371f6a7,
    0xf81deeffea561b71,
    0xdb60aaebf1759338,
    0xd881d38001c7df75,
    0x6073bdfcaf427c14,
    0x299bf463417881e1,
    0x048b490e1ec9a225,
    0x44572d90540b656e,
    0x8034f0e6d766b675,
    0xbe547417487848fa,
    0xa496fbd03e8c360d,
    0x759fd2c3100b14fc,
    0x818e0bbc6877234b,
    0x7104bf1f24d45ffa,
    0x5317341cfde65400,
    0xdb503cd59146b422,
    0x1e166b2c0819c3e1,
    0xa1e4c584687f358a,
    0x4df18d9cff51db02,
    0x263eeceb47c9ef8b,
    0xaaeb6bb721cf9ec6,
    0x5eb32dc7a3e78b93,
    0x612df6d79e2e93c4,
    0x171774f16f216aa4,
    0x0c91f6aa7acc317a,
    0x53447c8443d6a8fe,
    0xeb25ac150e713216,
    0xdeb3267b663c119e,
    0x7e4c6569befa9d16,
    0x3a679900c13355e2,
    0x74b6f99193981dbc,
    0x34b8a862356c11e3,
    0xd498bac2ef255f6e,
    0x625c90d64423b4b6,
    0x06a2947bc56d074e,
    0x20a3341acec39650,
    0xe4f1100dee9961d8,
    0xf61503329a84a0c8,
    0x4dc96a60a69da7d3,
    0x0948161917b35964,
    0x8ab254fbb28f37bb,
    0x79499910654e5d45,
    0x94de69e9d71ac8c3,
    0xfd16afd40b6f0e0f,
    0xc857bbe5214ebe29,
    0x6b194fe34e94b768,
    0xa1a443cc93c3efe3,
    0x3982176e89472193,
    0x347583a9ca350909,
    0xdb262692c2186abe,
    0x5cfed672ad62ddfa,
    0xed5f101a74becee8,
    0xc0dba50ffecc54fe,
    0x4bbd985a89cf2341,
    0xada88561755eda7f,
    0xfe878ed7cec1f330,
    0x88e2379e6a125e03,
    0x21e1638f70af563b,
    0xead693b8badf6ef3,
    0x517f7fd1de7fa605,
    0xea9b3bbf4fff43e2,
    0xe903631cf12049d3,
    0x864a53365ea38d5e,
    0x3ec4371f512d946a,
    0x5cbde66debb4104b,
    0xfae9344401aa6d7f,
    0xb2f4b146d9378d23,
    0xb76f06ae579565c5,
    0x24608c19df085f61,
    0x3f930f4991867f4f,
    0x06125d5f8bce0e9f,
    0xebdeb8774d788588,
    0xd3dc88925c07e639,
    0xbe0bd5c39857ab62,
    0x9f1c35a434cc5ecf,
    0x92a7b3096896e254,
    0x1147b07fe51975bf,
    0x4a7391f816b07d82,
    0xe730f72a04d74450,
    0xfd02b218c5372067,
    0x2998c34917a65d85,
    0x01f99342ee684e05,
    0x4ab624d26454e620,
    0xb5db70dc4268f0e3,
    0xfa1a9b771b44d71c,
    0x0f1d0ba66f9ccd77,
    0x8d34b1a9eafa6b57,
];
