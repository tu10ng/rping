# rping

simple ping program written for homework assignment, under development.

## Usage
use the `build.sh` script:
``` shell
./build.sh www.example.com
```

or manually:

``` shell
cargo build --release
sudo setcap cap_net_raw=+eip $(PATH_TO_PROGRAM)
cargo run --release -- www.example.com
```
## examples
``` shell
./build.sh help
./build.sh -4 -c 3 127.0.0.1
./build.sh -6 ::1

```
## problems
- -b flag:

when pinging 255.255.255.255, program panics without usefull information.

trying sudo instead of setcap have the same result.

no relative information found on the internet.

- thread::sleep stuck \C-c

- slower than `ping` program:

while running 

``` shell
ping www.example.com -c 5
```

ping gives 4000ms execution time, while our program gives 5000ms.

# TODO
- [x] send and receive packet
- [x] echo output message to STDOUT
- [ ] make ping and echo concurrent
- [x] calculate statistics after \C-c
- [ ] ~~colorful output message, maybe use crate `colored`~~
- args
  - [x] support ip address 
  - [x] support hostname
  - [ ] -b 允许ping一个广播地址，只用于IPv4
  - [x] -c 数目 在发送指定数日的包后停止
  - [x] -h 显示帮助信息
  - [x] -i 设定间隔几秒发送一个包给指定机器
  - [x] -q 安静模式，不显示每个收到的包的分析结果，只在结束时，显示汇总结果
  - [x] -s 指定发送的数据字节数
  - [x] -t 设置ttl值，只用于IPv
  - [x] -4 只ping ipv4地址
  - [x] -6 只ping ipv6地址
