# Changelog

All notable changes to this project will be documented in this file. See [standard-version](https://github.com/conventional-changelog/standard-version) for commit guidelines.

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