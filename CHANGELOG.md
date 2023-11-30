# Changelog

All notable changes to this project will be documented in this file. See [standard-version](https://github.com/conventional-changelog/standard-version) for commit guidelines.

### [0.2.11](https://github.com/nonconvextech/ftlog/compare/v0.2.10...v0.2.11) (2023-11-30)


### Features

* add `LoggerGuard` to ensure flush when exits ([fdb302d](https://github.com/nonconvextech/ftlog/commit/fdb302db53acaee314ce778b5523284d69d3710c))

### [0.2.10](https://github.com/nonconvextech/ftlog/compare/v0.2.9...v0.2.10) (2023-09-28)


### Features

* `FileAppender` builder, and timezone support ([792d6bc](https://github.com/nonconvextech/ftlog/commit/792d6bc57467e4f0dd8791df464ac9fbf03d6e51))
* add feature gate `random_drop` ([5c6de2d](https://github.com/nonconvextech/ftlog/commit/5c6de2d4d83870e6b99aa11e758fd979a8fc9e39))
* simultaneous multi destination helper ([5ec6e4c](https://github.com/nonconvextech/ftlog/commit/5ec6e4cff499a0641faad08caccd14693a3ab7da))

### [0.2.9](https://github.com/nonconvextech/ftlog/compare/v0.2.8...v0.2.9) (2023-08-25)


### Features

* do not write empty log message ([1054305](https://github.com/nonconvextech/ftlog/commit/105430582be5562473fbf28ade0d0f75e57b7a77))

### [0.2.8](https://github.com/nonconvextech/ftlog/compare/v0.2.7...v0.2.8) (2023-08-18)


### Features

* allow randomly drop log message ([61ceac2](https://github.com/nonconvextech/ftlog/commit/61ceac22d05b1c80b885a1fe6e11292432854e75))

### [0.2.7](https://github.com/nonconvextech/ftlog/compare/v0.2.6...v0.2.7) (2023-06-14)


### Features

* use `time` v0.3 format mechanism ([eb4c9d4](https://github.com/nonconvextech/ftlog/commit/eb4c9d42082c93752cc812b20960ff966fec1573))

### [0.2.6](https://github.com/nonconvextech/ftlog/compare/v0.2.5...v0.2.6) (2023-02-07)


### Bug Fixes

* file rotation at local timezone ([ea40194](https://github.com/nonconvextech/ftlog/commit/ea40194255f0a2a209e00d14bdc70b51ce6a0510))

### [0.2.5](https://github.com/nonconvextech/ftlog/compare/v0.2.4...v0.2.5) (2023-02-05)


### Features

* add `try_init(self)` for `ftlog::Builder` ([8a2e482](https://github.com/nonconvextech/ftlog/commit/8a2e48262fc8db410c2b9d501b46005967e40eb6))
* add feature `tsc` to use TSC as clock source ([0e9947e](https://github.com/nonconvextech/ftlog/commit/0e9947e3de861bb8c95e344723f418ea30d24e50))
* early stop when if msg in queue when flush ([32df7ec](https://github.com/nonconvextech/ftlog/commit/32df7ec25023fa79a0a9fd4aa80e16250de5af2e))
* option to use utc timestamp for performance ([864c021](https://github.com/nonconvextech/ftlog/commit/864c02111747c1bb26e04706acc290807f3ca4a3))
* timezone format to hour in log timestamp ([6b15626](https://github.com/nonconvextech/ftlog/commit/6b15626784bbc916d60b97ebf78b49506e86bf49))
* write msg in queue and flush when quit ([359d787](https://github.com/nonconvextech/ftlog/commit/359d787bfd99a240632da34e2aa8c2c99056622a))


### Bug Fixes

* flush write all messages in receiver queue ([2bf6081](https://github.com/nonconvextech/ftlog/commit/2bf6081e45acdc2e41d2520912c2816c221b1e9d))
* rotation can cause panic in windows ([d3dbf0d](https://github.com/nonconvextech/ftlog/commit/d3dbf0dc7845f3d9fba0bbdedf66e65f9b6a8e89))

### [0.2.4](https://github.com/nonconvextech/ftlog/compare/v0.2.3...v0.2.4) (2022-12-09)


### Bug Fixes

* root level does not follow global level ([04eb7bb](https://github.com/nonconvextech/ftlog/commit/04eb7bbb8fea4343b045a1ad6a8f24ef3265bafa))

### [0.2.3](https://github.com/nonconvextech/ftlog/compare/v0.2.2...v0.2.3) (2022-12-08)


### Features

* change default format style ([3d03497](https://github.com/nonconvextech/ftlog/commit/3d034977333a7ede7279b63f19c0e4fbac84cb73))

### [0.2.2](https://github.com/nonconvextech/ftlog/compare/v0.2.1...v0.2.2) (2022-12-05)


### Features

* avoid panic in main/worker thread ([1365e06](https://github.com/nonconvextech/ftlog/commit/1365e06a93e5b681c5fbbbc6a13b3f57bcbaf27c))
* avoid periodically check for flush ([460b843](https://github.com/nonconvextech/ftlog/commit/460b8433dd4a3342ee3fadd77ef012857c0595a2))

### [0.2.1](https://github.com/nonconvextech/ftlog/compare/v0.2.0...v0.2.1) (2022-11-25)


### Features

* add level filter for root appender ([635e5a5](https://github.com/nonconvextech/ftlog/commit/635e5a50e0b4b4667387a7698fdbe841ce7f09b3))

## [0.2.0](https://github.com/nonconvextech/ftlog/compare/v0.1.1...v0.2.0) (2022-11-22)


### Features

* add option to print total number of omitted log when set to discard excessive message with a minimal interval of 5s
* add example for simple case & custom format ([356af7d](https://github.com/nonconvextech/ftlog/commit/356af7d17e961506f4eb00be505b4a0de4fcad7b))
* allow control for bounded/unbound channel ([0712a83](https://github.com/nonconvextech/ftlog/commit/0712a837117dff14d28b682109a61d2b7fd479ea))
* clean expire log for rotate file appender ([961e39d](https://github.com/nonconvextech/ftlog/commit/961e39d1dc083e1a636563e05e21b45b008bb114))
* filters by module path ([b884ef7](https://github.com/nonconvextech/ftlog/commit/b884ef72d49cd8bf9fb47559ec3e4523cd076432))
* force flush every seconds ([1aed0d8](https://github.com/nonconvextech/ftlog/commit/1aed0d8f565c2763c7c3a0545eaa23ac4cb08531))
* re-export `log::logger` ([19334f1](https://github.com/nonconvextech/ftlog/commit/19334f1f661b3f7e707ff96022cd28f9c20f4da7))
* re-export log macros, LevelFilter and Record ([727fe3d](https://github.com/nonconvextech/ftlog/commit/727fe3daf546d51cad2c6352395cd2fdc5d1250d))
* remove outdated logs before log rotation ([1ca2175](https://github.com/nonconvextech/ftlog/commit/1ca21759d9683b11ec7daa4279523d96cb9a33da))
* use `log::info` to log deleted files ([6036354](https://github.com/nonconvextech/ftlog/commit/60363541816a4b00c84227f68906f37d62a72cce))
* warn incorrect level setting for filters ([f77f1fc](https://github.com/nonconvextech/ftlog/commit/f77f1fc5408b7bcb1ebe5df6d90a662ef945f126))


### Bug Fixes

* **appender:** avoid remove log file generated by others ([3251300](https://github.com/nonconvextech/ftlog/commit/3251300f48a0bd5eda63b0a40de5f49e457e776f))
* check outdated log when rotate new files ([0b8c32a](https://github.com/nonconvextech/ftlog/commit/0b8c32a4d308acf40acd663ef1e71071444f0258))
* flush all appenders ([9a46bb4](https://github.com/nonconvextech/ftlog/commit/9a46bb41e57ccbe66fd3c29b940408eb6ea9585b))

### [0.1.1](https://github.com/nonconvextech/ftlog/compare/v0.1.0...v0.1.1) (2022-11-01)


### Bug Fixes

* zero pad datetime ([1bf8df0](https://github.com/nonconvextech/ftlog/commit/1bf8df093d73a97605d256a0faa7b1a4a7597985))

## 0.1.0(2022-10-27)


### Features

* allow split file for different period: minute, hour, day, month, year
* change LogBuilder pattern to move builder
* log the count of dumped record when using `limit`

### Perf
* reduce usage of time related methods, otherwise will impact performance (especially in concurrency context)
* when using `limit` and in case of excessive call of `info!` and etc, improve the speed of channel consumer