import random

from locust import task, HttpUser


class AccountUser(HttpUser):
    endpoints = [
        "/accounts/{stake_address}",
        "/accounts/{stake_address}/rewards",
        "/accounts/{stake_address}/history",
        "/accounts/{stake_address}/delegations",
        "/accounts/{stake_address}/registrations",
        "/accounts/{stake_address}/withdrawals",
        "/accounts/{stake_address}/mirs",
        "/accounts/{stake_address}/addresses",
        "/accounts/{stake_address}/addresses/assets",
        "/accounts/{stake_address}/addresses/total",
    ]

    params = [
        {
            "stake_address": "stake1uxggdly3z7ed4k6rmg0eytzxqwdy04gmluy5x0wd6x83ensq3rs22"
        },
        {
            "stake_address": "stake1uy03wfpwwujl2lnznpar3r3l3ytghacgdr5kkxtf54xpjucw5k6rd"
        },
        {
            "stake_address": "stake1u8pu4tfaguufvstja6wex7nvvngqnmc4mcv3ewjq8t0l66qzs5jft"
        },
        {
            "stake_address": "stake1uxz29q5a2flp5l3cu857390gvrf43jr3gqsdm74pefs2nxqn32jya"
        },
        {
            "stake_address": "stake1u9f9v0z5zzlldgx58n8tklphu8mf7h4jvp2j2gddluemnssjfnkzz"
        },
    ]

    @task
    def home(self):
        endpoint = random.choice(self.endpoints)
        self.client.get(
            endpoint.format(**random.choice(self.params)),
            verify=False,
            name=endpoint,
        )


class AddressUser(HttpUser):
    endpoints = [
        # Addresses
        "/addresses/{address}",
        "/addresses/{address}/extended",
        "/addresses/{address}/total",
        "/addresses/{address}/utxos",
        "/addresses/{address}/txs",
    ]

    params = [
        {
            "address": "addr1q8l7hny7x96fadvq8cukyqkcfca5xmkrvfrrkt7hp76v3qvssm7fz9ajmtd58ksljgkyvqu6gl23hlcfgv7um5v0rn8qtnzlfk"
        },
        {
            "address": "addr1q8lm7czefzwcwm4gp4g9akzs4qyuhlfe9mzczptxzuuvwdglzujzuae974lx9xr68z8rlzgk30mss68fdvvknf2vr9esq6zl0j"
        },
        {
            "address": "addr1q9lyw9xtg7w4vv5ngchl9ppnx60z6u25kgnn067d6ax02awre2kn63ecjeqh9m5ajdaxcexsp8h3thserjayqwkll45q7yj04k"
        },
        {
            "address": "addr1q9n0cfjg76kursm668yvl9hn2r54qvj7wgj992zz4tzq53uy52pf65n7rflr3c0faz27scxntry8zspqmha2rjnq4xvqluh95t"
        },
        {
            "address": "addr1zxn9efv2f6w82hagxqtn62ju4m293tqvw0uhmdl64ch8uw6j2c79gy9l76sdg0xwhd7r0c0kna0tycz4y5s6mlenh8pq6s3z70"
        },
    ]

    @task
    def home(self):
        endpoint = random.choice(self.endpoints)
        self.client.get(
            endpoint.format(**random.choice(self.params)),
            verify=False,
            name=endpoint,
        )


class AssetUser(HttpUser):
    endpoints = [
        "/assets?count=10",
        "/assets/{asset}",
        "/assets/{asset}/history",
        "/assets/{asset}/txs",
        "/assets/{asset}/addresses",
    ]

    params = [
        {
            "asset": "00000002df633853f6a47465c9496721d2d5b1291b8398016c0e87ae6e7574636f696e"
        },
        {
            "asset": "3a9241cd79895e3a8d65261b40077d4437ce71e9d7c8c6c00e3f658e4669727374636f696e"
        },
        {"asset": "02f68378e37af4545d027d0a9fa5581ac682897a3fc1f6d8f936ed2b4154414441"},
        {"asset": "e8e62d329e73190190c3e323fb5c9fb98ee55f0676332ba949f29d724649525354"},
        {"asset": "ac3f4224723e2ed9d166478662f6e48bae9ddf0fc5ee58f54f6c322943454e54"},
        {"asset": "12e65fa3585d80cba39dcf4f59363bb68b77f9d3c0784734427b151754534c41"},
        {
            "asset": "e12ab5cf12f95cd57b739282d06af9dd61e1b1dde1e06f0c31f0251167696d62616c"
        },
        {"asset": "da8c30857834c6ae7203935b89278c532b3995245295456f993e1d244c51"},
        {
            "asset": "b863bc7369f46136ac1048adb2fa7dae3af944c3bbb2be2f216a8d4f4265727279416c6261"
        },
        {
            "asset": "b863bc7369f46136ac1048adb2fa7dae3af944c3bbb2be2f216a8d4f4265727279416d657468797374"
        },
        {
            "asset": "b863bc7369f46136ac1048adb2fa7dae3af944c3bbb2be2f216a8d4f4265727279417175616d6172696e65"
        },
        {
            "asset": "b863bc7369f46136ac1048adb2fa7dae3af944c3bbb2be2f216a8d4f4265727279417368"
        },
        {
            "asset": "b863bc7369f46136ac1048adb2fa7dae3af944c3bbb2be2f216a8d4f426572727941756275726e"
        },
        {
            "asset": "b863bc7369f46136ac1048adb2fa7dae3af944c3bbb2be2f216a8d4f4265727279417572656c6961"
        },
        {
            "asset": "b863bc7369f46136ac1048adb2fa7dae3af944c3bbb2be2f216a8d4f4265727279417572656f6c696e"
        },
        {
            "asset": "b863bc7369f46136ac1048adb2fa7dae3af944c3bbb2be2f216a8d4f4265727279417765736f6d65"
        },
        {
            "asset": "b863bc7369f46136ac1048adb2fa7dae3af944c3bbb2be2f216a8d4f4265727279417a756c"
        },
        {
            "asset": "b863bc7369f46136ac1048adb2fa7dae3af944c3bbb2be2f216a8d4f42657272794265696765"
        },
        {
            "asset": "b863bc7369f46136ac1048adb2fa7dae3af944c3bbb2be2f216a8d4f4265727279426572796c"
        },
        {
            "asset": "b863bc7369f46136ac1048adb2fa7dae3af944c3bbb2be2f216a8d4f4265727279426c61636b"
        },
    ]

    @task
    def home(self):
        endpoint = random.choice(self.endpoints)
        self.client.get(
            endpoint.format(**random.choice(self.params)),
            name=endpoint,
            verify=False,
        )


class BlockUser(HttpUser):
    endpoints = [
        "/blocks/latest",
        "/blocks/latest/txs",
        "/blocks/{hash_or_number}",
        "/blocks/{hash_or_number}/next",
        "/blocks/{hash_or_number}/previous?count=5",
        "/blocks/{hash_or_number}/txs",
        "/blocks/{hash_or_number}/addresses",
    ]

    params = [
        {
            "hash_or_number": "4e0ca338c882fe10a835f438605adfa72fd3e8d22768f319d7236014da2c920b"
        },
        {
            "hash_or_number": "3bbb6e88953e37b8006a34351d8a42053348c642d0784f60b13d3f97a3178c00"
        },
        {
            "hash_or_number": "f4a379f3cc397e297ed52088e3223232d555d7121daa93896e08b1132b7036b6"
        },
        {
            "hash_or_number": "256263f1cf9bf0e4d2de3c7c1a02a73c3ca9e4d8827876a0536cc8c142351bd0"
        },
        {
            "hash_or_number": "2757fd737e80c1ea791db1d6fc8d23d1f7a89b552beb0500306c45fe6a2226ed"
        },
        {
            "hash_or_number": "5501d69f77cd858cfa39f6a0e511451fd6533ade31235f8f044b3cd1c269a5a5"
        },
        {
            "hash_or_number": "d730d0ff535e2b7d1dc472f75c5e0a6583ddc4b76c91ab515dd30dc24822e81f"
        },
        {
            "hash_or_number": "2b3d35517fac7dd20d7a9897224939a90913e345e759f17bf253f7f5422e2713"
        },
        {
            "hash_or_number": "a2de214cd3546a02fae4fa7e680d030739fe55364a5c2fa06d8e896d87d6e402"
        },
        {
            "hash_or_number": "5eb8f400749597c1be0b537a2f6b9e0000da2dbbf5d837feb032c32457150770"
        },
    ]

    @task
    def home(self):
        endpoint = random.choice(self.endpoints)
        self.client.get(
            endpoint.format(**random.choice(self.params)),
            verify=False,
            name=endpoint,
        )


class EpochUser(HttpUser):
    endpoints = [
        "/epochs/latest",
        "/epochs/latest/parameters",
        "/epochs/{number}",
        "/epochs/{number}/next",
        "/epochs/{number}/previous?count=5",
        "/epochs/{number}/stakes",
        "/epochs/{number}/blocks",
        "/epochs/{number}/parameters",
    ]

    params = [
        {"number": 401},
        {"number": 402},
        {"number": 403},
        {"number": 404},
        {"number": 405},
        {"number": 406},
        {"number": 407},
        {"number": 408},
        {"number": 409},
        {"number": 410},
    ]

    @task
    def home(self):
        endpoint = random.choice(self.endpoints)
        self.client.get(
            endpoint.format(**random.choice(self.params)),
            verify=False,
            name=endpoint,
        )


class PoolUser(HttpUser):
    endpoints = [
        "/pools?count=10",
        "/pools/{pool_id}/history",
        "/pools/{pool_id}/metadata",
        "/pools/{pool_id}/relays",
        "/pools/{pool_id}/delegators",
        "/pools/{pool_id}/blocks",
        "/pools/{pool_id}/updates",
    ]

    params = [
        {"pool_id": "pool1z5uqdk7dzdxaae5633fqfcu2eqzy3a3rgtuvy087fdld7yws0xt"},
        {"pool_id": "pool1pu5jlj4q9w9jlxeu370a3c9myx47md5j5m2str0naunn2q3lkdy"},
        {"pool_id": "pool1q80jjs53w0fx836n8g38gtdwr8ck5zre3da90peuxn84sj3cu0r"},
        {"pool_id": "pool1ddskftmsscw92d7vnj89pldwx5feegkgcmamgt5t0e4lkd7mdp8"},
        {"pool_id": "pool1qqqqqdk4zhsjuxxd8jyvwncf5eucfskz0xjjj64fdmlgj735lr9"},
        {"pool_id": "pool1dmqzwuql5mylffvn7ln3pr9j7kh4gdsssrmma5wgx56f6rtyf42"},
        {"pool_id": "pool1qzlw7z5mutmd39ldyjnp8n650weqe55z5p8dl3fagac3ge0nx8l"},
        {"pool_id": "pool16tcjctesjnks0p8sfrlf8f3d3vrp2fdn2msy80sgg3cdjtayu3z"},
        {"pool_id": "pool1qzlwlpcsgflr9z3f24fg836tyq45p0kf5cnrp20s8v0psp6tdkx"},
        {"pool_id": "pool1p9sda64t6l9802tsu2fj6phvt9xfqgcpjucyr3kek8wzurmn8rz"},
    ]

    @task
    def home(self):
        endpoint = random.choice(self.endpoints)
        self.client.get(
            endpoint.format(**random.choice(self.params)),
            verify=False,
            name=endpoint,
        )


class TransactionUser(HttpUser):
    endpoints = [
        "/txs/{hash}",
        "/txs/{hash}/utxos",
        "/txs/{hash}/stakes",
        "/txs/{hash}/delegations",
        "/txs/{hash}/withdrawals",
        "/txs/{hash}/mirs",
        "/txs/{hash}/pool_updates",
        "/txs/{hash}/pool_retires",
        "/txs/{hash}/metadata",
        "/txs/{hash}/metadata/cbor",
        "/txs/{hash}/redeemers",
    ]

    params = [
        {"hash": "7a7646924fbc0491f7ea7a346741e3c37bcd35e1d00a608c924aa8f7df24e18c"},
        {"hash": "7476690702d93c464eee728d20f31e68964016b5c4dfec432e8002ca5f1016f7"},
        {"hash": "f4724af263c2a164655e45dd5103acf39bf42b6fbcaa5d76ab1efb5e01aec2b1"},
        {"hash": "5569bea9023b17d93348159a1b5ccd80f6c7f3d0af319fd3f906832a84a5ac49"},
        {"hash": "bf7d1291ff44e448ee3b664d7f46786d5f89878669be1337c2e8b6805aaa1cb5"},
        {"hash": "e07bfddbb5dfbeea3ca347e6e0f6eeb3b4bdfb47a3723019dfa68ca4d8d068c1"},
        {"hash": "ba28720abc31ae4d3a58df3c921452fabff7e8c69181490308394fd0030769bb"},
        {"hash": "da895cb7c4d4845d1311ef6306488fcea9ab6f43deeaeaeac147128910b32d3c"},
        {"hash": "1b20ed068ba22517585e93c14b004afe9ecf2f9af2c444e8922491bd33d58775"},
        {"hash": "643f10330979d398cbf802f1cdf9025841d2a59d3a78fc235ab17b87ee72c413"},
        {"hash": "55075ff41d7a4a530077d190d9c62c33a760608f714677af71a57abc5432ed22"},
        {"hash": "cf7836f6590432b984093ac5b1838333a687375e4e6910122f97d0d86b5b05b7"},
        {"hash": "0144ab5a9c813665097f7c8eea731421adb09c28039ab8a68a0e15ab76911aed"},
        {"hash": "899c7a59077b0b7d9de1ada769a70d2e27f04dac99063deb42d5e92727cf7613"},
        {"hash": "f199831b090c7f42539f5ea3d9577e40e38b47a4358544da0e03cb422352889c"},
        {"hash": "56e2a57a57664ecf4600d958a1052ae72379f3baec9685160058fd5c5ddf7ab1"},
        {"hash": "677bedb1a980f61f321316008390823910dd20c01e131ea3394b3789d38980fc"},
        {"hash": "0088bf97d97e0b07d9c951a041167903dbd70e1eb8dafd45f062c03c8a43bc88"},
    ]

    @task
    def home(self):
        endpoint = random.choice(self.endpoints)
        self.client.get(
            endpoint.format(**random.choice(self.params)),
            verify=False,
            name=endpoint,
        )


class ScriptUser(HttpUser):
    endpoints = [
        "/scripts",
        "/scripts/{script_hash}",
        "/scripts/{script_hash}/json",
        "/scripts/{script_hash}/cbor",
        "/scripts/{script_hash}/redeemers",
    ]

    params = [
        {"script_hash": "65c197d565e88a20885e535f93755682444d3c02fd44dd70883fe89e"},
        {"script_hash": "00000002df633853f6a47465c9496721d2d5b1291b8398016c0e87ae"},
        {"script_hash": "3a9241cd79895e3a8d65261b40077d4437ce71e9d7c8c6c00e3f658e"},
        {"script_hash": "02f68378e37af4545d027d0a9fa5581ac682897a3fc1f6d8f936ed2b"},
        {"script_hash": "e8e62d329e73190190c3e323fb5c9fb98ee55f0676332ba949f29d72"},
        {"script_hash": "ac3f4224723e2ed9d166478662f6e48bae9ddf0fc5ee58f54f6c3229"},
        {"script_hash": "12e65fa3585d80cba39dcf4f59363bb68b77f9d3c0784734427b1517"},
        {"script_hash": "e12ab5cf12f95cd57b739282d06af9dd61e1b1dde1e06f0c31f02511"},
        {"script_hash": "da8c30857834c6ae7203935b89278c532b3995245295456f993e1d24"},
        {"script_hash": "b863bc7369f46136ac1048adb2fa7dae3af944c3bbb2be2f216a8d4f"},
    ]

    @task
    def home(self):
        endpoint = random.choice(self.endpoints)
        self.client.get(
            endpoint.format(**random.choice(self.params)),
            verify=False,
            name=endpoint,
        )
