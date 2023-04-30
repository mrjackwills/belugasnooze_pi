### Chores
+ dependencies updated, [d319aaa58abc954125fa64c35fca13f966f03166]

# <a href='https://github.com/mrjackwills/belugasnooze_pi/releases/tag/v0.3.1'>v0.3.1</a>
### 2023-03-10

### Chores
+ Rust 1.68.0 linting, [ca2a085f](https://github.com/mrjackwills/belugasnooze_pi/commit/ca2a085f39a6540f69c2d970c03583300f093f73)
+ _typos.toml added, [18a49742](https://github.com/mrjackwills/belugasnooze_pi/commit/18a49742cde6b6a503458046658e4d5fcb7c089b)
+ create_release updated, [67aa26ea](https://github.com/mrjackwills/belugasnooze_pi/commit/67aa26eab028b7cad9fa130433fe5bd14665861d)
+ dependencies updated, [5356ed82](https://github.com/mrjackwills/belugasnooze_pi/commit/5356ed821941bae0564402cf33715324db6a2d95)
+ devcontainer updated, use sparse protocol index, [9f0fb6cd](https://github.com/mrjackwills/belugasnooze_pi/commit/9f0fb6cd06d69c9b72dc535eabb3bda9125fdf5e), [77c04170](https://github.com/mrjackwills/belugasnooze_pi/commit/77c041707f0a0657262b59c1d9512c76c5c8739f)
+ typos fixed, [1f576fe7](https://github.com/mrjackwills/belugasnooze_pi/commit/1f576fe7ab32f452b6edbaad4c8bc3b829e1a21e)

### Refactors
+ `unwrap_or(())` changed to `ok()`, [558fb205](https://github.com/mrjackwills/belugasnooze_pi/commit/558fb20589f7c23349ccba5814a8ca45e62b9e89)

# <a href='https://github.com/mrjackwills/belugasnooze_pi/releases/tag/v0.3.0'>v0.3.0</a>
### 2023-02-02

### Chores
+ dependencies updated, [dc5f837d](https://github.com/mrjackwills/belugasnooze_pi/commit/dc5f837d7c1fa7c93241c0d2d96482ae2289ade8)
+ dev container mount /dev/shm, [0ff14f5d](https://github.com/mrjackwills/belugasnooze_pi/commit/0ff14f5d007f1fe8c3d7672a8ceb746ae8b08398)
+ github action on main semver only, [013f87f6](https://github.com/mrjackwills/belugasnooze_pi/commit/013f87f68761cbc6c782c18a20a1757bd28d7835)

## Features
+ remove openssl dependency, [ba3ea537](https://github.com/mrjackwills/belugasnooze_pi/commit/ba3ea537aaf9d1a3f0f867d78cbb4e7e998c6455)
+ use a scratch Docker container, [fb844b56](https://github.com/mrjackwills/belugasnooze_pi/commit/fb844b56859117bd0740233d79e9e3c418f98be9)

### Refactors
+ linting, [82965e9b](https://github.com/mrjackwills/belugasnooze_pi/commit/82965e9baced9117e6e439703d20b27e0a126716)

# <a href='https://github.com/mrjackwills/belugasnooze_pi/releases/tag/v0.2.2'>v0.2.2</a>
### 2023-01-24

### Chores
+ dependencies updated, [4e9c84ec](https://github.com/mrjackwills/belugasnooze_pi/commit/4e9c84ec73cc25bf265dc8941fc080dfdc924d0c)

### Features
+ token request add timeout & useragent, [e598f6f6](https://github.com/mrjackwills/belugasnooze_pi/commit/e598f6f6ca8dd8feafff0aaa0edb268b05d13fa2)

# <a href='https://github.com/mrjackwills/belugasnooze_pi/releases/tag/v0.2.1'>v0.2.1</a>
### 2023-01-06

### Chores
+ dependencies updated, [3036cf7a](https://github.com/mrjackwills/belugasnooze_pi/commit/3036cf7a50168d7df9a696cedf7d88843451755f), [11bd312c](https://github.com/mrjackwills/belugasnooze_pi/commit/11bd312c08d3310454f145c16bfccfcdc2d9848d)

### Docs
+ comments & typos, [8c0235aa](https://github.com/mrjackwills/belugasnooze_pi/commit/8c0235aac0d72e1a03dc9c0c144093ccf37506cc)

### Features
+ log tests, [aec755e0](https://github.com/mrjackwills/belugasnooze_pi/commit/aec755e0fbcdaeda8a2676f66367860dd12408c9)
+ tracing log level into app_env, [740ef13f](https://github.com/mrjackwills/belugasnooze_pi/commit/740ef13f80c8a5ab496474b570235013f0df4055)

### Refactors
+ Dockerfile tweak, [b6b4603e](https://github.com/mrjackwills/belugasnooze_pi/commit/b6b4603e5dd85d41ac02316e53376f6df745b677)
+ lightstatus use relaxed ordering, [58100be2](https://github.com/mrjackwills/belugasnooze_pi/commit/58100be227b48bcc80bcf9e8825c6421ed7b7032)
+ is_connected removed, [9446df66](https://github.com/mrjackwills/belugasnooze_pi/commit/9446df66bc7c734a629973c78c97d9efccffc963)

# <a href='https://github.com/mrjackwills/belugasnooze_pi/releases/tag/v0.2.0'>v0.2.0</a>
### 2022-12-15

### Chores
+ lint with Rust 1.66, [af7229ac](https://github.com/mrjackwills/belugasnooze_pi/commit/af7229ac8e60a6f25e3e9f918c6335f1e807b0e6)
+ dependencies updated, [59867ee9](https://github.com/mrjackwills/belugasnooze_pi/commit/59867ee9cb6ceb1f4d1c93ab28f511b5014fa4c6), [0e0006a2](https://github.com/mrjackwills/belugasnooze_pi/commit/0e0006a25f3ad464ecfcea21754e3dddf29c7a52)

### Docs
+ workflow comments, [faff6b06](https://github.com/mrjackwills/belugasnooze_pi/commit/faff6b06bc023bce8809e49a263387bdae92c92e)

### Features
+ use EnvTimeZone, [ccd287da](https://github.com/mrjackwills/belugasnooze_pi/commit/ccd287dacf1dbff6c78bdfe932b6fd25ff9de891)
+ github action cache, [5497cfd1](https://github.com/mrjackwills/belugasnooze_pi/commit/5497cfd1a299d1b03effea52593c53d378509ecf)

### Fixes
+ docker-compose typo, [7d9f76fe](https://github.com/mrjackwills/belugasnooze_pi/commit/7d9f76fe75f2ece9d308b70c1876a36edc229a0b)

### Refactors
+ use saturating_sub in alarm loop to calculate time to sleep for, [29db696b](https://github.com/mrjackwills/belugasnooze_pi/commit/29db696bd457b62553d55d7dc77bc1359b9e1523)

# <a href='https://github.com/mrjackwills/belugasnooze_pi/releases/tag/v0.1.9'>v0.1.9</a>
### 2022-11-23

### Chores
+ use dtolnay/rust-toolchain in github workflow, [37ca2413](https://github.com/mrjackwills/belugasnooze_pi/commit/37ca24139480fbd1701330f1f5790cbf7972e88b)
+ dead code removed, [2c4e0d45](https://github.com/mrjackwills/belugasnooze_pi/commit/2c4e0d4524676a68e1e8939df1809ba286a77044)
+ docker container alpine v3.17, [f34bcf84](https://github.com/mrjackwills/belugasnooze_pi/commit/f34bcf84800db6c1ce220fbcfc4a43b8eec47d80)
+ dependencies updated, rand removed, [f74664e7](https://github.com/mrjackwills/belugasnooze_pi/commit/f74664e74d0cf20a52681a2c3d7ba372821faa84)

# <a href='https://github.com/mrjackwills/belugasnooze_pi/releases/tag/v0.1.8'>v0.1.8</a>
### 2022-11-13

### Fix
+ create_release.sh fix, [a3098998](https://github.com/mrjackwills/belugasnooze_pi/commit/a3098998bb588835ceb1a189c078a5e85ceffb14)

# <a href='https://github.com/mrjackwills/belugasnooze_pi/releases/tag/v0.1.7'>v0.1.7</a>
### 2022-11-13

### Chores
+ update rust, aggressive linting, [89b28a17](https://github.com/mrjackwills/belugasnooze_pi/commit/89b28a175958fe7890cfd9799956d41ab2b5732c)
+ create_release.sh v0.1.0, [2682f563](https://github.com/mrjackwills/belugasnooze_pi/commit/2682f563610c64c61b13355ed19d04d1adbb74a6)

### Features
+ remove stored offset, calculate each time needed, [05777779](https://github.com/mrjackwills/belugasnooze_pi/commit/05777779e575afb79ffdc7b122c2056093e811f6)

### Fixes
+ create_release.sh duplicate comma, [e8ff6748](https://github.com/mrjackwills/belugasnooze_pi/commit/e8ff674806d462b4052be58a8da30cf167959abe)

### Refactors
+ dead code removed, [c38b4dff](https://github.com/mrjackwills/belugasnooze_pi/commit/c38b4dff59e09e4c277795e2ad39968e92bf646f)

# <a href='https://github.com/mrjackwills/belugasnooze_pi/releases/tag/v0.1.6'>v0.1.6</a>
### 2022-10-12

### Fixes
+ Rebuild due to tag issue

# <a href='https://github.com/mrjackwills/belugasnooze_pi/releases/tag/v0.1.5'>v0.1.5</a>
### 2022-10-12

### Chores
+ create_release.sh sleep to enable Cargo.lock to update, [4cf84590](https://github.com/mrjackwills/belugasnooze_pi/commit/4cf84590972ac8fa14c273d71ced4656e8cd15a0),

### Refactors
+ auto_close remove internal messaging, [c2d645f0](https://github.com/mrjackwills/belugasnooze_pi/commit/c2d645f0a8b6572f1823f0d14b1ac0fdb1cedf92),

# <a href='https://github.com/mrjackwills/belugasnooze_pi/releases/tag/v0.1.4'>v0.1.4</a>
### 2022-10-12

### Chores
+ Cargo.lock tracked, [3fe9a8db](https://github.com/mrjackwills/belugasnooze_pi/commit/3fe9a8db697cf8d06dc8f13963ba0d12f5bde400),
+ dev container install cross, [4e900f34](https://github.com/mrjackwills/belugasnooze_pi/commit/4e900f3419eb88b33018ea612e227f1c2800f33d),

### Features
+ autoclose after 40 seconds in no ping received, [70991a5c](https://github.com/mrjackwills/belugasnooze_pi/commit/70991a5c9df08d3526b5da589012993a541e02ea),

# <a href='https://github.com/mrjackwills/belugasnooze_pi/releases/tag/v0.1.3'>v0.1.3</a>
### 2022-10-08

### Chores
+ dev container updated, [81b917a9](https://github.com/mrjackwills/belugasnooze_pi/commit/81b917a9c4029cda3a3cbb2c59cd3f39f3c92d70),

### Refactors
+ dead code removed, [24c0578d](https://github.com/mrjackwills/belugasnooze_pi/commit/24c0578db006b6e6684a10a5bd236c44da0d0535),
+ into_iter().map, [522185e6](https://github.com/mrjackwills/belugasnooze_pi/commit/522185e65844caefac9c53ae86e4873e455a3dc7),
+ remove async from parse_env, [66f7f9aa](https://github.com/mrjackwills/belugasnooze_pi/commit/66f7f9aaa0128cc164d873253f06bf025eb6b2e8),


# <a href='https://github.com/mrjackwills/belugasnooze_pi/releases/tag/v0.1.2'>v0.1.2</a>
### 2022-09-27

### Fixes
+ .env typo fix, [848a4a6b](https://github.com/mrjackwills/belugasnooze_pi/commit/848a4a6b6fd5ce3ef80393bbfec190c1978e6290),

# <a href='https://github.com/mrjackwills/belugasnooze_pi/releases/tag/v0.1.1'>v0.1.1</a>
### 2022-09-27

### Chores
+ .gitignore updated, [5002bd5c](https://github.com/mrjackwills/belugasnooze_pi/commit/5002bd5c5ad720c03233195ef91e23d0b293a8e5),

### Features
+ read env from a docker Read Only mount, or local.env, [cc0adf60](https://github.com/mrjackwills/belugasnooze_pi/commit/cc0adf608fdd8637b0b9644accfdc61a6ddaab4f),

# <a href='https://github.com/mrjackwills/belugasnooze_pi/releases/tag/v0.1.0'>v0.1.0</a>
### 2022-09-27

### Docs
+ readme updated, [55ca5424](https://github.com/mrjackwills/belugasnooze_pi/commit/55ca5424f59a66bcb73843eff574cd58a69996a4),
+ readme.md typo, [58706612](https://github.com/mrjackwills/belugasnooze_pi/commit/587066128e868d82657229642af05ce08930e04c),

### Chores
+ anyhow import removed, [e2982862](https://github.com/mrjackwills/belugasnooze_pi/commit/e2982862903d96b4e2de95b182c6adcdbcb69ff0),
+ dependencies updated, [a332282d](https://github.com/mrjackwills/belugasnooze_pi/commit/a332282d32fc2d4181f1171d58f5728388282561), [07e54204](https://github.com/mrjackwills/belugasnooze_pi/commit/07e542049d1567e20a7d57350454ed7c8f753419),

### Features
+ use new staticpi protocol, [c335e530](https://github.com/mrjackwills/belugasnooze_pi/commit/c335e530f70cbf1c803552be64f766bff152ef6b),
+ use AppError for all errors, [5ae8a63c](https://github.com/mrjackwills/belugasnooze_pi/commit/5ae8a63c4d485ce5993152e559cf6125ff053376),
+ anyhow removed, AppError enum used, [1a29406f](https://github.com/mrjackwills/belugasnooze_pi/commit/1a29406f7786ac375d59128f0a186fc4bcb2e939),

### Fixes
+ create_release update version numbers & Dockefile location, [72cd427e](https://github.com/mrjackwills/belugasnooze_pi/commit/72cd427e14b29d7e55ebdbcf81d127ff141ce88f),
+ sql connection settings updated, mod sql > mod db, [23ad29c7](https://github.com/mrjackwills/belugasnooze_pi/commit/23ad29c7e583574f9333419590618f3de3cc8239),

### Refactors
+ dead code removed, [45b1b648](https://github.com/mrjackwills/belugasnooze_pi/commit/45b1b648d35f6f758ab0323b1681defb4c20c66e),
+ incoming_ws_message match & consume, [9db74ac1](https://github.com/mrjackwills/belugasnooze_pi/commit/9db74ac114822bba640aab4e99486b12f34239ee),
+ rainbow colors made a const, [fe4b7951](https://github.com/mrjackwills/belugasnooze_pi/commit/fe4b79511d75830789506012e83d77cb40e84340),
+ location_logs removed, [1a751a46](https://github.com/mrjackwills/belugasnooze_pi/commit/1a751a4610c89166bbae66ff316cb2dd0d1ac094),

# <a href='https://github.com/mrjackwills/belugasnooze_pi/releases/tag/v0.0.2'>v0.0.2</a>
### 2022-08-02

### Features
+ Dockerfile download binary from github, [64908669](https://github.com/mrjackwills/belugasnooze_pi/commit/6490866919bc36640cdbc5fb33ca9a7ba5b9d895),

### Fixes
+ Aggressive linting, [82b2ea1b](https://github.com/mrjackwills/belugasnooze_pi/commit/82b2ea1b8a0effa7264e66e050c5ffb7555571ac),


# <a href='https://github.com/mrjackwills/belugasnooze_pi/releases/tag/v0.0.1'>v0.0.1</a>
### 2022-05-30

### Features

+ init commit
