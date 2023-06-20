# rping

simple ping program written for homework assignment, under development.

## Usage
set capability manually:

``` shell
sudo setcap cap_net_raw=+eip $(PATH_TO_PROGRAM)
```

# TODO
- [ ] send and receive packet
- [ ] echo output message to STDOUT
- [ ] calculate statistics after \C-c
- [ ] colorful output message, maybe use crate `colored`
- [ ] -b 允许ping一个广播地址，只用于IPv4
- [ ] -c 数目 在发送指定数日的包后停止
- [x] -h 显示帮助信息
- [ ] -i 设定间隔几秒发送一个包给指定机器
- [ ] -q 安静模式，不显示每个收到的包的分析结果，只在结束时，显示汇总结果
- [ ] -s 指定发送的数据字节数
- [ ] -t 设置ttl值，只用于IPv
