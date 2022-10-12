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
+ dependenices updated, [a332282d](https://github.com/mrjackwills/belugasnooze_pi/commit/a332282d32fc2d4181f1171d58f5728388282561), [07e54204](https://github.com/mrjackwills/belugasnooze_pi/commit/07e542049d1567e20a7d57350454ed7c8f753419),

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
